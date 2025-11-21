use std::fs::Metadata;

pub fn to_string(metadata: Metadata, _is_dir: bool, _is_symlink: bool) -> String {
    #[cfg(unix)]
    {
        unix_permissions(metadata, _is_dir, _is_symlink)
    }

    #[cfg(windows)]
    {
        windows_attributes(metadata)
    }
}

#[cfg(unix)]
fn unix_permissions(
    metadata: Metadata,
    is_dir: bool,
    is_symlink: bool,
) -> String {
    let mut permission_str = String::new();

    use std::os::unix::fs::MetadataExt;

    let perms = metadata.mode() & 0o777;

    permission_str.push(match (is_dir, is_symlink) {
        (true, false) => 'd',
        (false, true) => 'l',
        _ => '-',
    });

    for i in (0..3).rev() {
        let bits = (perms >> (i * 3)) & 0b111;
        permission_str.push(if bits & 0b100 != 0 { 'r' } else { '-' });
        permission_str.push(if bits & 0b010 != 0 { 'w' } else { '-' });
        permission_str.push(if bits & 0b001 != 0 { 'x' } else { '-' });
    }

    permission_str
}

#[cfg(windows)]
fn windows_attributes(metadata: Metadata) -> String {
    use std::os::windows::fs::MetadataExt;

    let mut permission_str = String::new();
    let attrs = metadata.file_attributes();

    const FILE_ATTRIBUTE_DIRECTORY: u32 = 16;
    const FILE_ATTRIBUTE_ARCHIVE: u32 = 32;
    const FILE_ATTRIBUTE_READONLY: u32 = 1;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 2;
    const FILE_ATTRIBUTE_SYSTEM: u32 = 4;

    let options = vec![
        (FILE_ATTRIBUTE_DIRECTORY, 'd'),
        (FILE_ATTRIBUTE_ARCHIVE, 'a'),
        (FILE_ATTRIBUTE_READONLY, 'r'),
        (FILE_ATTRIBUTE_HIDDEN, 'h'),
        (FILE_ATTRIBUTE_SYSTEM, 's'),
    ];

    options.iter().for_each(|option| {
        let ch = if attrs & option.0 != 0 { option.1 } else { '-' };
        permission_str.push(ch);
    });

    permission_str
}
