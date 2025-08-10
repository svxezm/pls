use std::fs::Metadata;

pub fn to_string(metadata: Metadata, is_dir: bool, is_symlink: bool) -> String {
    let mut permission_str = String::new();

    #[cfg(unix)]
    {
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
    {
        use std::os::windows::fs::MetadataExt;
        use winapi::um::winnt::*;

        let add = |target: &mut String, valid: bool, char1: char, char2: char| {
            target.push(if valid { char1 } else { char2 });
        };

        let perms = metadata.file_attributes();

        permission_str.push(match (is_dir, is_symlink) {
            (true, false) => 'd',
            (false, true) => 'l',
            _ => '-',
        });

        if perms & FILE_ATTRIBUTE_NORMAL != 0 {
            permission_str.push_str("---n-");
        } else {
            add(
                &mut permission_str,
                perms & FILE_ATTRIBUTE_READONLY != 0,
                'r',
                '-',
            );
            add(
                &mut permission_str,
                perms & FILE_ATTRIBUTE_HIDDEN != 0,
                'h',
                '-',
            );
            add(
                &mut permission_str,
                perms & FILE_ATTRIBUTE_SYSTEM != 0,
                's',
                '-',
            );
            add(
                &mut permission_str,
                perms & FILE_ATTRIBUTE_ARCHIVE != 0,
                'a',
                '-',
            );
            permission_str.push('-');
        }

        permission_str
    }
}
