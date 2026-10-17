use types::language::Language;

use super::{guide, quoted, zsh_script};
use crate::arguments::full_command;
use crate::texts::Texts;

fn french() -> Texts {
    Texts::new(Language::French)
}

#[test]
fn the_script_is_a_zsh_completion_function_for_batdetect() {
    assert!(zsh_script(french()).starts_with("#compdef batdetect\n"));
}

#[test]
fn every_long_option_of_the_full_help_is_completed() {
    let script = zsh_script(french());
    for argument in full_command(french()).get_arguments() {
        if let Some(long) = argument.get_long() {
            let specified = [format!("--{long}["), format!("--{long}=[")];
            assert!(specified.iter().any(|spec| script.contains(spec)), "--{long}");
        }
    }
}

#[test]
fn the_zoom_offers_its_three_values() {
    assert!(zsh_script(french()).contains(":(auto all none)'"));
}

#[test]
fn the_inputs_repeat_and_offer_only_videos_of_the_five_extensions_and_folders_never_hidden_files() {
    let script = zsh_script(french());
    assert!(
        script.contains(r#"    _files -g "[^.]*.(#i)(mp4|mov|avi|mkv|m4v)" && ret=0"#),
        "{script}"
    );
    assert!(script.contains(":_batdetect_inputs' \\\n"), "{script}");
    assert!(script.contains("'*:inputs -- "), "{script}");
    assert!(!script.contains("':inputs -- "), "{script}");
}

#[test]
fn doctor_and_completion_are_offered_with_their_description_only_as_the_first_word() {
    let script = zsh_script(french());
    assert!(script.contains("    if (( CURRENT == 2 )); then\n"), "{script}");
    assert!(
        script.contains(
            "commands=('doctor:vérifie que tout est prêt avant une soirée' 'completion:écrit la complétion de la touche Tab')"
        ),
        "{script}"
    );
    let defined = script.find("_batdetect_inputs() {").unwrap();
    assert!(defined < script.find("    _batdetect \"$@\"").unwrap(), "{script}");
}

#[test]
fn a_description_with_an_apostrophe_stays_inside_its_zsh_quotes() {
    assert_eq!(quoted("l'aide"), r"'l'\''aide'");
}

#[test]
fn the_output_folder_completes_folders_only() {
    assert!(zsh_script(french()).contains(":_files -/' \\"));
}

#[test]
fn the_guide_gives_the_compinit_line_with_homebrew_and_the_three_steps_without() {
    let guide = guide(french());
    assert_eq!(guide.matches("autoload -Uz compinit && compinit").count(), 2);
    assert!(guide.contains("mkdir -p ~/.zfunc && batdetect completion zsh > ~/.zfunc/_batdetect"));
    assert!(guide.contains("fpath=(~/.zfunc $fpath)"));
    assert!(guide.find("fpath=").unwrap() < guide.rfind("compinit").unwrap());
}
