use crate::manifests::installer_manifest::Scope;
use crate::types::architecture::Architecture;

pub const VALID_FILE_EXTENSIONS: [&str; 7] = [
    "msix",
    "msi",
    "appx",
    "exe",
    "zip",
    "msixbundle",
    "appxbundle",
];

const DELIMITERS: [char; 6] = [',', '/', '\\', '.', '_', '-'];

const ARCHITECTURES: [(&str, Architecture); 29] = [
    ("x86_64", Architecture::X64),
    ("x64", Architecture::X64),
    ("64-bit", Architecture::X64),
    ("64bit", Architecture::X64),
    ("win64", Architecture::X64),
    ("winx64", Architecture::X64),
    ("ia64", Architecture::X64),
    ("amd64", Architecture::X64),
    ("x86", Architecture::X86),
    ("x32", Architecture::X86),
    ("32-bit", Architecture::X86),
    ("32bit", Architecture::X86),
    ("win32", Architecture::X86),
    ("winx86", Architecture::X86),
    ("ia32", Architecture::X86),
    ("i386", Architecture::X86),
    ("i486", Architecture::X86),
    ("i586", Architecture::X86),
    ("i686", Architecture::X86),
    ("386", Architecture::X86),
    ("486", Architecture::X86),
    ("586", Architecture::X86),
    ("686", Architecture::X86),
    ("arm64", Architecture::Arm64),
    ("aarch64", Architecture::Arm64),
    ("arm", Architecture::Arm),
    ("armv7", Architecture::Arm),
    ("aarch", Architecture::Arm),
    ("neutral", Architecture::Neutral),
];

pub fn find_architecture(url: &str) -> Option<Architecture> {
    // Ignore the casing of the URL
    let url = url.to_lowercase();

    // Check for {delimiter}{architecture}{delimiter}
    for (arch_name, arch) in ARCHITECTURES {
        if url.contains(arch_name) {
            let mut url_chars = url.chars();
            // Get the character before the architecture, consuming the characters before it
            let char_before_arch = url
                .rfind(arch_name)
                .and_then(|arch_index| url_chars.nth(arch_index - 1));
            // As the characters have been consumed, we can skip by the length of the architecture
            let char_after_arch = url_chars.nth(arch_name.chars().count());
            // If the architecture is surrounded by valid delimiters, the architecture is valid
            if DELIMITERS.contains(&char_before_arch.unwrap_or_default())
                && DELIMITERS.contains(&char_after_arch.unwrap_or_default())
            {
                return Some(arch);
            }
        }
    }

    // If the architecture has not been found, check for {architecture}.{extension}
    let extensions = VALID_FILE_EXTENSIONS
        .iter()
        .filter(|&extension| url.contains(extension));
    for extension in extensions {
        for (arch_name, arch) in ARCHITECTURES {
            if url.contains(&format!("{arch_name}.{extension}")) {
                return Some(arch);
            }
        }
    }

    None
}

pub fn find_scope(url: &str) -> Option<Scope> {
    match url.to_ascii_lowercase() {
        url if url.contains("user") => Some(Scope::User),
        url if url.contains("machine") => Some(Scope::Machine),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::types::architecture::Architecture;
    use crate::url_utils::find_architecture;
    use rstest::rstest;

    #[rstest]
    fn test_x64_architectures_at_end(
        #[values("x86_64", "x64", "64-bit", "64bit", "Win64", "Winx64", "ia64", "amd64")]
        architecture: &str,
    ) {
        assert_eq!(
            find_architecture(&format!("https://www.example.com/file{architecture}.exe")),
            Some(Architecture::X64)
        );
    }

    #[rstest]
    fn test_x64_architectures_delimited(
        #[values("x86_64", "x64", "64-bit", "64bit", "Win64", "Winx64", "ia64", "amd64")]
        architecture: &str,
        #[values(',', '/', '\\', '.', '_', '-')] delimiter: char,
    ) {
        assert_eq!(
            find_architecture(&format!(
                "https://www.example.com/file{delimiter}{architecture}{delimiter}app.exe"
            )),
            Some(Architecture::X64)
        );
    }

    #[rstest]
    fn test_x86_architectures_at_end(
        #[values(
            "x86", "x32", "32-bit", "32bit", "win32", "winx86", "ia32", "i386", "i486", "i586",
            "i686", "386", "486", "586", "686"
        )]
        architecture: &str,
    ) {
        assert_eq!(
            find_architecture(&format!("https://www.example.com/file{architecture}.exe")),
            Some(Architecture::X86)
        );
    }

    #[rstest]
    fn test_x86_architectures_delimited(
        #[values(
            "x86", "x32", "32-bit", "32bit", "win32", "winx86", "ia32", "i386", "i486", "i586",
            "i686", "386", "486", "586", "686"
        )]
        architecture: &str,
        #[values(',', '/', '\\', '.', '_', '-')] delimiter: char,
    ) {
        assert_eq!(
            find_architecture(&format!(
                "https://www.example.com/file{delimiter}{architecture}{delimiter}app.exe"
            )),
            Some(Architecture::X86)
        );
    }

    #[rstest]
    fn test_arm64_architectures_at_end(#[values("arm64", "aarch64")] architecture: &str) {
        assert_eq!(
            find_architecture(&format!("https://www.example.com/file{architecture}.exe")),
            Some(Architecture::Arm64)
        );
    }

    #[rstest]
    fn test_arm64_architectures_delimited(
        #[values("arm64", "aarch64")] architecture: &str,
        #[values(',', '/', '\\', '.', '_', '-')] delimiter: char,
    ) {
        assert_eq!(
            find_architecture(&format!(
                "https://www.example.com/file{delimiter}{architecture}{delimiter}app.exe"
            )),
            Some(Architecture::Arm64)
        );
    }

    #[rstest]
    fn test_arm_architectures_at_end(#[values("arm", "armv7", "aarch")] architecture: &str) {
        assert_eq!(
            find_architecture(&format!("https://www.example.com/file{architecture}.exe")),
            Some(Architecture::Arm)
        );
    }

    #[rstest]
    fn test_arm_architectures_delimited(
        #[values("arm", "armv7", "aarch")] architecture: &str,
        #[values(',', '/', '\\', '.', '_', '-')] delimiter: char,
    ) {
        assert_eq!(
            find_architecture(&format!(
                "https://www.example.com/file{delimiter}{architecture}{delimiter}app.exe"
            )),
            Some(Architecture::Arm)
        );
    }

    #[test]
    fn test_no_architecture() {
        assert_eq!(find_architecture("https://www.example.com/file.exe"), None);
    }
}
