use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;
use crate::update_state::UpdateState;
use clap::{crate_name, crate_version};
use rand::{thread_rng, Rng};
use std::env;
use uuid::Uuid;

pub fn get_package_path(
    identifier: &PackageIdentifier,
    version: Option<&PackageVersion>,
) -> String {
    let first_character = identifier
        .chars()
        .next()
        .map(|mut first_character| {
            first_character.make_ascii_lowercase();
            first_character
        })
        .unwrap();
    let mut result = format!("manifests/{first_character}");
    for part in identifier.split('.') {
        result.push('/');
        result.push_str(part);
    }
    if let Some(version) = version {
        result.push('/');
        result.push_str(&version.to_string());
    }
    result
}

pub fn get_pull_request_body() -> String {
    const FRUITS: [&str; 16] = [
        "apple",
        "banana",
        "blueberries",
        "cherries",
        "grapes",
        "green_apple",
        "kiwi_fruit",
        "lemon",
        "mango",
        "melon",
        "peach",
        "pear",
        "pineapple",
        "strawberry",
        "tangerine",
        "watermelon",
    ];

    let custom_tool_info = if let (Ok(tool_name), Ok(tool_url)) = (
        env::var("KOMAC_CREATED_WITH"),
        env::var("KOMAC_CREATED_WITH_URL"),
    ) {
        format!("[{tool_name}]({tool_url})")
    } else {
        format!(
            "[{}]({}) v{}",
            crate_name!(),
            env!("CARGO_PKG_REPOSITORY"),
            crate_version!()
        )
    };

    let mut rng = thread_rng();

    let emoji = if rng.gen_range(0..50) == 0 {
        FRUITS[rng.gen_range(0..FRUITS.len())]
    } else {
        "rocket"
    };

    format!("### Pull request has been created with {custom_tool_info} :{emoji}:")
}

pub fn get_branch_name(
    package_identifier: &PackageIdentifier,
    package_version: &PackageVersion,
) -> String {
    /// GitHub rejects branch names longer than 255 bytes. Considering `refs/heads/`, 244 bytes are left for the name.
    const MAX_BRANCH_NAME_LEN: usize = u8::MAX as usize - "refs/heads/".len();
    let mut uuid_buffer = Uuid::encode_buffer();
    let uuid = Uuid::new_v4().simple().encode_upper(&mut uuid_buffer);
    let mut branch_name = format!("{package_identifier}-{package_version}-{uuid}");
    if branch_name.len() > MAX_BRANCH_NAME_LEN {
        branch_name.truncate(MAX_BRANCH_NAME_LEN - uuid.len());
        branch_name.push_str(uuid);
    }
    branch_name
}

pub fn get_commit_title(
    identifier: &PackageIdentifier,
    version: &PackageVersion,
    update_state: &UpdateState,
) -> String {
    format!("{update_state}: {identifier} version {version}")
}

#[cfg(test)]
mod tests {
    use crate::github::utils::get_package_path;
    use crate::types::package_identifier::PackageIdentifier;
    use crate::types::package_version::PackageVersion;

    #[test]
    fn test_partial_package_path() {
        let identifier = PackageIdentifier::parse("Package.Identifier").unwrap_or_default();
        assert_eq!(
            "manifests/p/Package/Identifier",
            get_package_path(&identifier, None)
        );
    }

    #[test]
    fn test_full_package_path() {
        let identifier = PackageIdentifier::parse("Package.Identifier").unwrap_or_default();
        let version = PackageVersion::new("1.2.3").unwrap_or_default();
        assert_eq!(
            "manifests/p/Package/Identifier/1.2.3",
            get_package_path(&identifier, Some(&version))
        );
    }
}
