//! The numbers and words every part of Trindle's buttons share.

pub const DISARM_SECONDS: u32 = 3;

pub const TICK_SECONDS: u32 = 2;
pub const TICK_ICON: &str = "object-select-symbolic";

pub fn click_again(name: &str) -> String {
    let mut chars = name.chars();
    let Some(first) = chars.next() else { return crate::t!("kit-click-again-bare").into() };
    let rest: String = chars.collect();
    let lower = crate::t!("kit-click-again-small") == "yes" && rest.chars().next().is_none_or(|c| !c.is_uppercase());
    let first = if lower { first.to_lowercase().collect::<String>() } else { first.to_string() };
    crate::t!("kit-click-again", action = format!("{first}{rest}"))
}

pub fn more_lines(lines: usize) -> String {
    match lines {
        0 => crate::t!("kit-more").into(),
        n => crate::t!("kit-more-lines", n = n),
    }
}

pub fn wait_while(job_runs: bool) -> Option<&'static str> {
    job_runs.then(|| crate::t!("kit-job-wait"))
}

pub const SAVE_ICON: &str = "media-floppy-symbolic";

#[cfg(test)]
mod tests {
    use super::click_again;

    #[test]
    fn an_armed_name_reads_on() {
        assert_eq!(click_again("Forget this network"), "Click again to forget this network");
        assert_eq!(click_again("Remove"), "Click again to remove");
        assert_eq!(click_again("USB stick off"), "Click again to USB stick off");
        assert_eq!(click_again("Delete, with their files"), "Click again to delete, with their files");
        assert_eq!(click_again("A"), "Click again to a");
        assert_eq!(click_again(""), "Click again");
    }

    #[test]
    fn a_language_that_keeps_its_capitals_keeps_them() {
        use crate::lang::{with_catalogue, Bundle, Catalogue, KIT_ENGLISH};
        let german = "kit-click-again = Noch einmal klicken: { $action }\nkit-click-again-small = no";
        let catalogue = Catalogue::new(Bundle::from_text("de", german), Bundle::from_files("en", KIT_ENGLISH));
        with_catalogue(catalogue, || assert_eq!(click_again("Netzwerk vergessen"), "Noch einmal klicken: Netzwerk vergessen"));
    }
}
