use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::imageops::FilterType;
use image::{DynamicImage, ImageEncoder};
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, FileType, TaggedFile, TaggedFileExt};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey, Tag, TagType};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Longest edge for embedded artwork (baseline JPEG).
const ARTWORK_MAX_PX: u32 = 800;
const ARTWORK_JPEG_QUALITY: u8 = 85;

const AUDIO_EXT: &[&str] = &[
    "mp3", "flac", "fla", "m4a", "m4b", "m4p", "m4r", "aac", "ogg", "opus", "wav", "aif", "aiff",
    "aifc", "afc", "wma", "ape", "mpc", "wv", "tta", "mp4", "3gp", "mov",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackRow {
    pub id: String,
    pub path: String,
    pub file_name: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub year: String,
    pub album_artist: String,
    pub track_number: String,
    pub tracks_total: String,
    pub comment: String,
    pub tag_format: String,
    pub dirty: bool,
    pub has_artwork: bool,
    pub pending_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArtworkChange {
    path: String,
    action: String,
    data_base64: Option<String>,
    #[allow(dead_code)] // kept for frontend payload shape; embed always re-encodes JPEG
    mime: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpResult {
    ok: usize,
    errors: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtworkPayload {
    mime: String,
    data_base64: String,
}

fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXT.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn collect_from_paths(paths: Vec<String>) -> Vec<PathBuf> {
    let mut out = HashSet::new();
    for p in paths {
        let path = PathBuf::from(&p);
        if path.is_file() && is_audio(&path) {
            out.insert(path);
        } else if path.is_dir() {
            for entry in WalkDir::new(&path)
                .follow_links(true)
                .into_iter()
                .filter_map(Result::ok)
            {
                let fp = entry.into_path();
                if fp.is_file() && is_audio(&fp) {
                    out.insert(fp);
                }
            }
        }
    }
    let mut sorted: Vec<_> = out.into_iter().collect();
    sorted.sort();
    sorted
}

/// Tag format label (e.g. `ID3v2.3.0`, `Vorbis`, `ID3v1, ID3v2.4.0`).
fn tag_format_label(path: &Path, tagged: &TaggedFile) -> String {
    let id3v2_ver = peek_id3v2_version(path);
    let mut parts = Vec::new();
    for tag in tagged.tags() {
        let label = match tag.tag_type() {
            TagType::Id3v1 => "ID3v1".to_string(),
            TagType::Id3v2 => id3v2_ver
                .clone()
                .unwrap_or_else(|| "ID3v2.4.0".to_string()),
            TagType::VorbisComments => "Vorbis".to_string(),
            TagType::Mp4Ilst => "MP4".to_string(),
            TagType::Ape => "APE".to_string(),
            TagType::RiffInfo => "Info".to_string(),
            TagType::AiffText => "AIFF".to_string(),
            _ => continue,
        };
        if !parts.iter().any(|p| p == &label) {
            parts.push(label);
        }
    }
    if parts.is_empty() {
        return match tagged.primary_tag_type() {
            TagType::VorbisComments => "Vorbis".to_string(),
            TagType::Mp4Ilst => "MP4".to_string(),
            TagType::RiffInfo => "Info".to_string(),
            TagType::Ape => "APE".to_string(),
            // ID3-capable (mp3/wav/aiff): prefer most common ID3v2.3
            _ => "ID3v2.3.0".to_string(),
        };
    }
    parts.join(", ")
}

fn peek_id3v2_version(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut buf = [0u8; 10];
    file.read_exact(&mut buf).ok()?;
    if &buf[0..3] != b"ID3" {
        return None;
    }
    Some(format!("ID3v2.{}.{}", buf[3], buf[4]))
}

fn first_nonempty(tags: &[Tag], f: impl Fn(&Tag) -> Option<String>) -> String {
    for tag in tags {
        if let Some(s) = f(tag) {
            if !s.trim().is_empty() {
                return s;
            }
        }
    }
    String::new()
}

/// When the file has no tags (common for Pro Tools WAV masters), derive
/// artist/title/track from patterns like `01 - Artist - Title` or
/// `Artist - A1 - Title - Master…`.
fn fill_from_filename(row: &mut TrackRow) {
    if !row.title.is_empty() && !row.artist.is_empty() {
        return;
    }
    let stem = Path::new(&row.file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(row.file_name.as_str());
    let parts: Vec<&str> = stem
        .split(" - ")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() < 2 {
        if row.title.is_empty() {
            row.title = stem.to_string();
        }
        return;
    }

    let looks_track = |s: &str| {
        !s.is_empty()
            && s.len() <= 4
            && s.chars()
                .all(|c| c.is_ascii_digit() || matches!(c, 'A'..='D' | 'a'..='d'))
            && s.chars().any(|c| c.is_ascii_digit())
    };

    // `01 - Artist - Title…`
    if looks_track(parts[0]) && parts.len() >= 3 {
        if row.track_number.is_empty() {
            row.track_number = parts[0]
                .trim_start_matches('0')
                .to_string();
            if row.track_number.is_empty() {
                row.track_number = parts[0].to_string();
            }
        }
        if row.artist.is_empty() {
            row.artist = parts[1].to_string();
        }
        if row.title.is_empty() {
            row.title = parts[2..].join(" - ");
        }
        return;
    }

    // `Artist - A1 - Title - Master…` or `Artist - Title`
    if row.artist.is_empty() {
        row.artist = parts[0].to_string();
    }
    if row.title.is_empty() {
        if parts.len() >= 3 && looks_track(parts[1]) {
            row.title = parts[2].to_string();
        } else {
            row.title = parts[1].to_string();
        }
    }
}

/// lofty 0.22 wrote ID3v2.3 APIC with encoding=UTF-16 but a Latin1 empty-desc terminator.
/// Patch those frames to Latin1 so Probe::read succeeds (image bytes stay intact).
fn repair_broken_apic_encoding(path: &Path) -> bool {
    let Ok(mut data) = fs::read(path) else {
        return false;
    };
    if data.len() < 10 || &data[..3] != b"ID3" {
        return false;
    }
    let ver = data[3];
    let tag_size = ((data[6] as usize & 0x7f) << 21)
        | ((data[7] as usize & 0x7f) << 14)
        | ((data[8] as usize & 0x7f) << 7)
        | (data[9] as usize & 0x7f);
    let end = (10 + tag_size).min(data.len());
    let mut i = 10usize;
    let mut changed = false;
    while i + 10 <= end {
        let id = &data[i..i + 4];
        if id.iter().all(|&b| b == 0) {
            break;
        }
        let fsize = if ver == 4 {
            ((data[i + 4] as usize & 0x7f) << 21)
                | ((data[i + 5] as usize & 0x7f) << 14)
                | ((data[i + 6] as usize & 0x7f) << 7)
                | (data[i + 7] as usize & 0x7f)
        } else {
            ((data[i + 4] as usize) << 24)
                | ((data[i + 5] as usize) << 16)
                | ((data[i + 6] as usize) << 8)
                | (data[i + 7] as usize)
        };
        let body = i + 10;
        let body_end = (body + fsize).min(data.len());
        if id == b"APIC" && body < body_end && data[body] == 1 {
            let mut j = body + 1;
            while j < body_end && data[j] != 0 {
                j += 1;
            }
            // enc | mime\0 | pic_type | desc...
            let desc = j + 2;
            if desc < body_end && data[desc] == 0 {
                let next = data.get(desc + 1).copied().unwrap_or(0);
                let is_png = next == 0x89;
                let is_jpeg = next == 0xFF && data.get(desc + 2) == Some(&0xD8);
                if is_png || is_jpeg {
                    data[body] = 0; // Latin1 matches the single-null empty description
                    changed = true;
                }
            }
        }
        if fsize == 0 {
            break;
        }
        i = body_end;
    }
    if changed {
        let _ = fs::write(path, &data);
    }
    changed
}

fn open_tagged(path: &Path) -> Result<TaggedFile, String> {
    repair_broken_apic_encoding(path);
    Probe::open(path)
        .map_err(|e| e.to_string())?
        .guess_file_type()
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())
}

fn read_track(path: &Path) -> Result<TrackRow, String> {
    let tagged = open_tagged(path)?;

    let tags = tagged.tags();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let mut row = TrackRow {
        id: path.to_string_lossy().to_string(),
        path: path.to_string_lossy().to_string(),
        file_name,
        title: first_nonempty(tags, |t| t.title().map(|s| s.to_string())),
        artist: first_nonempty(tags, |t| t.artist().map(|s| s.to_string())),
        album: first_nonempty(tags, |t| t.album().map(|s| s.to_string())),
        genre: first_nonempty(tags, |t| t.genre().map(|s| s.to_string())),
        year: first_nonempty(tags, |t| t.year().map(|n| n.to_string())),
        album_artist: first_nonempty(tags, |t| {
            t.get_string(&ItemKey::AlbumArtist).map(|s| s.to_string())
        }),
        track_number: first_nonempty(tags, |t| t.track().map(|n| n.to_string())),
        tracks_total: first_nonempty(tags, |t| t.track_total().map(|n| n.to_string())),
        comment: first_nonempty(tags, |t| t.comment().map(|s| s.to_string())),
        tag_format: tag_format_label(path, &tagged),
        dirty: false,
        has_artwork: tags.iter().any(|t| !t.pictures().is_empty()),
        pending_name: None,
    };
    fill_from_filename(&mut row);
    Ok(row)
}

fn first_picture(tag: Option<&Tag>) -> Option<&Picture> {
    tag.and_then(|t| {
        t.pictures()
            .iter()
            .find(|p| p.pic_type() == PictureType::CoverFront)
            .or_else(|| t.pictures().first())
    })
}

fn mime_from_ext(path: &Path) -> MimeType {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => MimeType::Png,
        Some("gif") => MimeType::Gif,
        Some("bmp") => MimeType::Bmp,
        _ => MimeType::Jpeg,
    }
}

fn ext_from_mime(mime: &MimeType) -> &str {
    match mime {
        MimeType::Png => "png",
        MimeType::Gif => "gif",
        MimeType::Bmp => "bmp",
        _ => "jpg",
    }
}

fn read_artwork_bytes(path: &Path) -> Result<ArtworkPayload, String> {
    let tagged = open_tagged(path)?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    let picture = first_picture(tag).ok_or_else(|| "No embedded artwork".to_string())?;
    let mime = picture
        .mime_type()
        .map(|m| format!("image/{}", ext_from_mime(&m)))
        .unwrap_or_else(|| "image/jpeg".to_string());
    Ok(ArtworkPayload {
        mime,
        data_base64: STANDARD.encode(picture.data()),
    })
}

fn prepare_embed_artwork(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let img = if img.width() > ARTWORK_MAX_PX || img.height() > ARTWORK_MAX_PX {
        img.resize(ARTWORK_MAX_PX, ARTWORK_MAX_PX, FilterType::Lanczos3)
    } else {
        img
    };
    encode_jpeg(&img)
}

fn encode_jpeg(img: &DynamicImage) -> Result<Vec<u8>, String> {
    let rgb = img.to_rgb8();
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, ARTWORK_JPEG_QUALITY)
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| e.to_string())?;
    Ok(out)
}

fn apply_artwork(tag: &mut Tag, change: &ArtworkChange) -> Result<(), String> {
    if change.action == "remove" {
        while !tag.pictures().is_empty() {
            tag.remove_picture(0);
        }
        return Ok(());
    }
    let data = change
        .data_base64
        .as_ref()
        .ok_or_else(|| "Missing artwork data".to_string())?;
    let raw = STANDARD
        .decode(data)
        .map_err(|e| e.to_string())?;
    let bytes = prepare_embed_artwork(&raw)?;
    tag.remove_picture_type(PictureType::CoverFront);
    // ponytail: lofty 0.22 None-desc + ID3v2.3 UTF-16 writes one 0x00; Some("") encodes properly. Drop when upgrading lofty ≥0.25.
    tag.push_picture(Picture::new_unchecked(
        PictureType::CoverFront,
        Some(MimeType::Jpeg),
        Some(String::new()),
        bytes,
    ));
    Ok(())
}

fn primary_tag_mut(tagged: &mut TaggedFile) -> &mut Tag {
    if tagged.primary_tag().is_some() {
        return tagged.primary_tag_mut().expect("primary tag exists");
    }
    let ty = tagged.primary_tag_type();
    let t = if ty != TagType::Id3v2 {
        tagged.insert_tag(Tag::new(ty));
        tagged.primary_tag_mut().unwrap()
    } else {
        tagged.insert_tag(Tag::new(TagType::Id3v2));
        tagged.primary_tag_mut().unwrap()
    };
    t
}

fn apply_row_to_tag(tag: &mut Tag, row: &TrackRow) {
    tag.set_title(row.title.trim().to_string());
    tag.set_artist(row.artist.trim().to_string());
    tag.set_album(row.album.trim().to_string());
    tag.set_genre(row.genre.trim().to_string());
    tag.set_comment(row.comment.trim().to_string());
    tag.insert_text(ItemKey::AlbumArtist, row.album_artist.trim().to_string());

    if let Ok(y) = row.year.trim().parse::<u32>() {
        tag.set_year(y);
    }
    if let Ok(n) = row.track_number.trim().parse::<u32>() {
        tag.set_track(n);
    }
    if let Ok(n) = row.tracks_total.trim().parse::<u32>() {
        tag.set_track_total(n);
    }
}

fn sanitize_filename(name: &str) -> String {
    let mut s = name
        .chars()
        .map(|c| match c {
            '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect::<String>();
    if s.len() > 180 {
        s.truncate(180);
    }
    s.trim().to_string()
}

fn track_token(row: &TrackRow, pad: bool) -> String {
    let t = row.track_number.trim();
    if let Ok(n) = t.parse::<u32>() {
        return if pad {
            format!("{n:02}")
        } else {
            n.to_string()
        };
    }
    if pad && t.len() == 1 {
        format!("0{t}")
    } else {
        t.to_string()
    }
}

fn expand_pattern(pattern: &str, row: &TrackRow, stem: &str, ext: &str, pad_track: bool) -> String {
    let track_pad = track_token(row, pad_track);
    let mut out = pattern.to_string();
    for (from, to) in [
        ("{{tracknumber}}", track_pad.as_str()),
        ("{{trackNumber}}", track_pad.as_str()),
        ("{{title}}", row.title.trim()),
        ("{{artist}}", row.artist.trim()),
        ("{{album}}", row.album.trim()),
        ("{{genre}}", row.genre.trim()),
        ("{{filename}}", stem),
        ("{{year}}", row.year.trim()),
        ("{{albumartist}}", row.album_artist.trim()),
    ] {
        out = out.replace(from, to);
    }
    let name = sanitize_filename(out.trim());
    if ext.is_empty() {
        name
    } else {
        format!("{name}.{ext}")
    }
}

#[tauri::command]
fn load_paths(paths: Vec<String>) -> Result<Vec<TrackRow>, String> {
    let files = collect_from_paths(paths);
    let mut rows = Vec::with_capacity(files.len());
    let mut errors = Vec::new();
    for path in files {
        match read_track(&path) {
            Ok(row) => rows.push(row),
            Err(e) => errors.push(format!("{}: {e}", path.display())),
        }
    }
    if rows.is_empty() && !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(rows)
}

#[tauri::command]
fn get_artwork(path: String) -> Result<ArtworkPayload, String> {
    read_artwork_bytes(Path::new(&path))
}

#[tauri::command]
fn load_artwork_file(source_path: String) -> Result<ArtworkPayload, String> {
    let path = Path::new(&source_path);
    if is_audio(path) {
        return read_artwork_bytes(path);
    }
    let data = fs::read(path).map_err(|e| e.to_string())?;
    let mime = match mime_from_ext(path) {
        MimeType::Png => "image/png",
        MimeType::Gif => "image/gif",
        MimeType::Bmp => "image/bmp",
        _ => "image/jpeg",
    };
    Ok(ArtworkPayload {
        mime: mime.to_string(),
        data_base64: STANDARD.encode(data),
    })
}

#[tauri::command]
fn export_artworks(paths: Vec<String>, dest_dir: String) -> OpResult {
    let dest = Path::new(&dest_dir);
    fs::create_dir_all(dest).map_err(|e| e.to_string()).ok();
    let mut ok = 0usize;
    let mut errors = Vec::new();
    for path in paths {
        let p = Path::new(&path);
        let stem = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("artwork");
        match read_artwork_bytes(p) {
            Ok(payload) => {
                let ext = payload
                    .mime
                    .split('/')
                    .nth(1)
                    .unwrap_or("jpg");
                let out = dest.join(format!("{stem}.{ext}"));
                let bytes = STANDARD.decode(&payload.data_base64).unwrap_or_default();
                match fs::write(&out, bytes) {
                    Ok(()) => ok += 1,
                    Err(e) => errors.push(format!("{}: {e}", path)),
                }
            }
            Err(e) => errors.push(format!("{}: {e}", path)),
        }
    }
    OpResult { ok, errors }
}

fn write_track(
    path: &Path,
    row: Option<&TrackRow>,
    artwork: Option<&ArtworkChange>,
) -> Result<(), String> {
    let mut tagged = open_tagged(path)?;
    {
        let tag = primary_tag_mut(&mut tagged);
        if let Some(row) = row {
            apply_row_to_tag(tag, row);
        }
        if let Some(change) = artwork {
            apply_artwork(tag, change)?;
        }
    }
    // WAV dual-write: ID3v2 (primary) + RIFF INFO for Rekordbox
    let want_info = row.is_some_and(|r| r.tag_format.contains("Info"));
    if want_info && tagged.file_type() == FileType::Wav {
        if tagged.tag(TagType::RiffInfo).is_none() {
            tagged.insert_tag(Tag::new(TagType::RiffInfo));
        }
        if let (Some(row), Some(info)) = (row, tagged.tag_mut(TagType::RiffInfo)) {
            apply_row_to_tag(info, row);
        }
    }
    let use_v23 = row
        .map(|r| r.tag_format.contains("ID3v2.3"))
        .unwrap_or(false);
    tagged
        .save_to_path(path, WriteOptions::new().use_id3v23(use_v23))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_tracks(tracks: Vec<TrackRow>, artwork_changes: Vec<ArtworkChange>) -> OpResult {
    let mut ok = 0usize;
    let mut errors = Vec::new();
    let mut art_by_path: std::collections::HashMap<String, &ArtworkChange> = artwork_changes
        .iter()
        .map(|c| (c.path.clone(), c))
        .collect();
    let mut touched = HashSet::new();

    for row in tracks.iter().filter(|r| r.dirty) {
        let path = Path::new(&row.path);
        let art = art_by_path.remove(&row.path);
        touched.insert(row.path.clone());
        match write_track(path, Some(row), art) {
            Ok(()) => ok += 1,
            Err(e) => errors.push(format!("{}: {e}", row.path)),
        }
    }

    for (path_str, change) in art_by_path {
        if touched.contains(&path_str) {
            continue;
        }
        match write_track(Path::new(&path_str), None, Some(change)) {
            Ok(()) => ok += 1,
            Err(e) => errors.push(format!("{path_str}: {e}")),
        }
    }

    OpResult { ok, errors }
}

#[tauri::command]
fn rename_tracks(tracks: Vec<TrackRow>, pattern: String, pad_track: bool) -> Result<Vec<TrackRow>, String> {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return Err("Rename pattern is empty".into());
    }
    let mut updated = tracks;
    let mut errors = Vec::new();
    for row in updated.iter_mut() {
        let path = Path::new(&row.path);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let new_name = expand_pattern(&pattern, row, &stem, &ext, pad_track);
        if new_name == row.file_name {
            row.pending_name = None;
            continue;
        }
        let parent = path.parent().ok_or_else(|| "No parent directory".to_string())?;
        let dest = parent.join(&new_name);
        if dest.exists() && dest != path {
            errors.push(format!("{} → {} (exists)", row.file_name, new_name));
            continue;
        }
        match fs::rename(path, &dest) {
            Ok(()) => {
                row.path = dest.to_string_lossy().to_string();
                row.id = row.path.clone();
                row.file_name = new_name;
                row.pending_name = None;
            }
            Err(e) => errors.push(format!("{}: {e}", row.file_name)),
        }
    }
    if !errors.is_empty() && errors.len() == updated.len() {
        return Err(errors.join("\n"));
    }
    Ok(updated)
}

#[tauri::command]
fn preview_rename(tracks: Vec<TrackRow>, pattern: String, pad_track: bool) -> Vec<TrackRow> {
    let pattern = pattern.trim();
    tracks
        .into_iter()
        .map(|mut row| {
            let path = Path::new(&row.path);
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let new_name = expand_pattern(&pattern, &row, &stem, &ext, pad_track);
            row.pending_name = if new_name != row.file_name {
                Some(new_name)
            } else {
                None
            };
            row
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn read_track_reads_common_id3_tags() {
        let p = Path::new("/Users/maxencerobinet/Music/PioneerDJ/Demo Tracks/Demo Track 1.mp3");
        if !p.is_file() {
            return;
        }
        let row = read_track(p).expect("read_track");
        assert_eq!(row.title, "Demo Track 1");
        assert_eq!(row.artist, "Loopmasters");
        assert!(!row.file_name.is_empty());
    }

    #[test]
    fn fill_from_filename_parses_master_and_numbered() {
        let mut a = TrackRow {
            id: String::new(),
            path: String::new(),
            file_name: "Milk & Honey - A1 - U Can Do It - Master Gio V2.wav".into(),
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            genre: String::new(),
            year: String::new(),
            album_artist: String::new(),
            track_number: String::new(),
            tracks_total: String::new(),
            comment: String::new(),
            tag_format: String::new(),
            dirty: false,
            has_artwork: false,
            pending_name: None,
        };
        fill_from_filename(&mut a);
        assert_eq!(a.artist, "Milk & Honey");
        assert_eq!(a.title, "U Can Do It");

        let mut b = a.clone();
        b.file_name = "01 - Milk & Honey - U Can Do it.mp3".into();
        b.title.clear();
        b.artist.clear();
        b.track_number.clear();
        fill_from_filename(&mut b);
        assert_eq!(b.track_number, "1");
        assert_eq!(b.artist, "Milk & Honey");
        assert_eq!(b.title, "U Can Do it");
    }

    #[test]
    fn write_digi_wav_roundtrip() {
        let p = Path::new("/tmp/te-empty-write.wav");
        assert!(p.is_file(), "fixture missing — copy a digi wav to /tmp/te-empty-write.wav");
        let tagged = Probe::open(p).unwrap().guess_file_type().unwrap().read().unwrap();
        eprintln!(
            "before: tags={} primary={:?}",
            tagged.tags().len(),
            tagged.primary_tag_type()
        );
        let mut row = read_track(p).expect("read");
        eprintln!("row format={:?} title={:?}", row.tag_format, row.title);
        row.title = "The Meeting".into();
        row.artist = "Milk & Honey".into();
        row.album = "The Honeymoon".into();
        row.genre = "Deep House".into();
        row.year = "2026".into();
        row.album_artist = "Milk & Honey".into();
        row.track_number = "3".into();
        row.tracks_total = "5".into();
        row.comment = "test".into();
        row.tag_format = "ID3v2.3.0".into();
        let err = write_track(p, Some(&row), None);
        eprintln!("write: {:?}", err);
        assert!(err.is_ok(), "write failed: {:?}", err.err());
        let again = read_track(p).expect("reread");
        eprintln!(
            "after: format={:?} title={:?} artist={:?}",
            again.tag_format, again.title, again.artist
        );
        assert_eq!(again.title, "The Meeting");
        assert_eq!(again.artist, "Milk & Honey");
    }

    #[test]
    fn write_wav_dual_id3_and_info() {
        let src = Path::new("/tmp/te-empty-write.wav");
        if !src.is_file() {
            return;
        }
        let dest = Path::new("/tmp/te-dual-write.wav");
        fs::copy(src, dest).expect("copy fixture");
        let mut row = read_track(dest).expect("read");
        row.title = "Dual Title".into();
        row.artist = "Dual Artist".into();
        row.album = "Dual Album".into();
        row.genre = "Deep House".into();
        row.comment = "rb".into();
        row.year = "2026".into();
        row.tag_format = "ID3v2.3.0, Info".into();
        write_track(dest, Some(&row), None).expect("dual write");

        let tagged = Probe::open(dest)
            .unwrap()
            .guess_file_type()
            .unwrap()
            .read()
            .unwrap();
        let types: Vec<_> = tagged.tags().iter().map(|t| t.tag_type()).collect();
        assert!(types.contains(&TagType::Id3v2), "missing ID3v2: {types:?}");
        assert!(types.contains(&TagType::RiffInfo), "missing Info: {types:?}");
        let info = tagged.tag(TagType::RiffInfo).expect("Info tag");
        assert_eq!(info.title().as_deref(), Some("Dual Title"));
        assert_eq!(info.artist().as_deref(), Some("Dual Artist"));
        assert_eq!(info.album().as_deref(), Some("Dual Album"));
    }

    #[test]
    fn save_tracks_five_digi() {
        let dir = Path::new(
            "/Users/maxencerobinet/Library/CloudStorage/GoogleDrive-maxencerobinet@gmail.com/Mon Drive/LABELS/CHUWANAGA/RELEASES/CHUWANAGA019/PROD/MASTER DIGI",
        );
        if !dir.is_dir() {
            return;
        }
        let mut tracks = Vec::new();
        for ent in fs::read_dir(dir).unwrap() {
            let ent = ent.unwrap();
            let p = ent.path();
            if p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("wav")) != Some(true) {
                continue;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('_') || !name.chars().next().unwrap_or('x').is_ascii_digit() {
                continue;
            }
            let mut row = read_track(&p).expect("read");
            row.dirty = true;
            row.album = "The Honeymoon".into();
            row.genre = "Deep House".into();
            tracks.push(row);
        }
        eprintln!("saving {} tracks", tracks.len());
        let result = save_tracks(tracks, vec![]);
        eprintln!("result ok={} errors={:?}", result.ok, result.errors);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert!(result.ok >= 5);
    }

    #[test]
    fn apic_png_description_terminator_is_utf16() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/silent.mp3");
        let png = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/1x1.png");
        assert!(src.is_file() && png.is_file(), "fixtures missing");
        let dest = Path::new("/tmp/te-apic-utf16.mp3");
        fs::copy(&src, dest).expect("copy");
        let png_bytes = fs::read(&png).expect("png");
        let mut row = read_track(dest).expect("read");
        row.tag_format = "ID3v2.3.0".into();
        let change = ArtworkChange {
            path: dest.display().to_string(),
            action: "set".into(),
            data_base64: Some(STANDARD.encode(png_bytes)),
            mime: Some("image/png".into()),
        };
        write_track(dest, Some(&row), Some(&change)).expect("write artwork");

        let data = fs::read(dest).expect("read back");
        assert_eq!(&data[..3], b"ID3");
        assert_eq!(data[3], 3, "expected ID3v2.3");
        let tag_size = ((data[6] as usize & 0x7f) << 21)
            | ((data[7] as usize & 0x7f) << 14)
            | ((data[8] as usize & 0x7f) << 7)
            | (data[9] as usize & 0x7f);
        let mut i = 10usize;
        let end = 10 + tag_size;
        let mut found = false;
        while i + 10 <= end {
            let id = &data[i..i + 4];
            let fsize = ((data[i + 4] as usize) << 24)
                | ((data[i + 5] as usize) << 16)
                | ((data[i + 6] as usize) << 8)
                | (data[i + 7] as usize);
            if id == b"APIC" {
                let body = &data[i + 10..i + 10 + fsize];
                assert_eq!(body[0], 1, "expected UTF-16 text encoding");
                let mime_end = body[1..].iter().position(|&b| b == 0).expect("mime nul");
                assert_eq!(&body[1..1 + mime_end], b"image/jpeg");
                let after_mime = 1 + mime_end + 1;
                assert_eq!(body[after_mime], 3, "picture type CoverFront");
                let desc = &body[after_mime + 1..];
                assert!(
                    desc.starts_with(&[0xFF, 0xFE, 0x00, 0x00])
                        || desc.starts_with(&[0xFE, 0xFF, 0x00, 0x00]),
                    "bad desc prefix: {:02x?}",
                    &desc[..desc.len().min(8)]
                );
                let img = &desc[4..];
                assert_eq!(&img[..2], &[0xFF, 0xD8], "embedded art must be JPEG");
                let decoded = image::load_from_memory(img).expect("jpeg decodes");
                assert!(decoded.width() <= 800 && decoded.height() <= 800);
                found = true;
                break;
            }
            if fsize == 0 || id.iter().all(|&b| b == 0) {
                break;
            }
            i += 10 + fsize;
        }
        assert!(found, "APIC frame missing");
    }

    #[test]
    fn prepare_embed_artwork_caps_at_800_jpeg() {
        let big = DynamicImage::new_rgb8(1600, 1200);
        let mut png = Vec::new();
        big.write_to(
            &mut std::io::Cursor::new(&mut png),
            image::ImageFormat::Png,
        )
        .unwrap();
        let out = prepare_embed_artwork(&png).expect("prepare");
        assert_eq!(&out[..2], &[0xFF, 0xD8]);
        let decoded = image::load_from_memory(&out).unwrap();
        assert_eq!(decoded.width(), 800);
        assert_eq!(decoded.height(), 600);
    }

    #[test]
    fn open_tagged_repairs_broken_apic_utf16() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/silent.mp3");
        let png = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/1x1.png");
        let dest = Path::new("/tmp/te-apic-broken.mp3");
        fs::copy(&src, dest).unwrap();
        let mut row = read_track(dest).unwrap();
        row.tag_format = "ID3v2.3.0".into();
        let change = ArtworkChange {
            path: dest.display().to_string(),
            action: "set".into(),
            data_base64: Some(STANDARD.encode(fs::read(&png).unwrap())),
            mime: Some("image/png".into()),
        };
        write_track(dest, Some(&row), Some(&change)).unwrap();

        // Corrupt: UTF-16 enc + Latin1 empty-desc terminator before JPEG
        let mut data = fs::read(dest).unwrap();
        let tag_size = ((data[6] as usize & 0x7f) << 21)
            | ((data[7] as usize & 0x7f) << 14)
            | ((data[8] as usize & 0x7f) << 7)
            | (data[9] as usize & 0x7f);
        let mut i = 10usize;
        let end = 10 + tag_size;
        while i + 10 <= end {
            let id = &data[i..i + 4];
            let fsize = ((data[i + 4] as usize) << 24)
                | ((data[i + 5] as usize) << 16)
                | ((data[i + 6] as usize) << 8)
                | (data[i + 7] as usize);
            if id == b"APIC" {
                let body = i + 10;
                assert_eq!(data[body], 1);
                let mut j = body + 1;
                while data[j] != 0 {
                    j += 1;
                }
                let desc = j + 2;
                // proper write is BOM+null (4 bytes); smash to single 0x00 + JPEG
                assert_eq!(&data[desc..desc + 2], &[0xFF, 0xFE]);
                data[desc] = 0;
                data[desc + 1] = 0xFF;
                data[desc + 2] = 0xD8;
                // leave rest; lofty only needs the prefix to fail
                break;
            }
            i += 10 + fsize;
        }
        fs::write(dest, &data).unwrap();
        assert!(
            Probe::open(dest)
                .unwrap()
                .guess_file_type()
                .unwrap()
                .read()
                .is_err(),
            "corrupt APIC should fail lofty"
        );
        let row = open_tagged(dest).expect("repair+open");
        assert!(
            row.tags().iter().any(|t| !t.pictures().is_empty()),
            "artwork readable after repair"
        );
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_paths,
            save_tracks,
            rename_tracks,
            preview_rename,
            get_artwork,
            load_artwork_file,
            export_artworks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}