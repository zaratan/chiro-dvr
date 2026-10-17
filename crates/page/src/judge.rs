use crate::escape::text;
use crate::labels::{Label, Labels};
use crate::model::Video;
use crate::script::Script;

#[cfg(test)]
mod tests;

pub(crate) fn render(video: &Video, labels: &dyn Labels) -> String {
    let say = |script: Script| text(&labels.label(&Label::Script(script)));
    let verdict = |class: &str, value: &str, word: Script, key: Script| {
        format!(
            r#"<button type="button" class="{class}" data-verdict="{value}" aria-pressed="false" aria-keyshortcuts="{key}">{word}<kbd aria-hidden="true">{key}</kbd></button>"#,
            word = say(word),
            key = say(key),
        )
    };
    format!(
        r#"      <div class="judge" hidden tabindex="-1">
        <div class="judge-head">
          <span class="who"><span class="dot" aria-hidden="true"></span><span class="now"></span></span>
          <span class="view" role="group" aria-label="{view}"><button type="button" class="view-clip" aria-pressed="true">{clip}</button><button type="button" class="view-zoom" aria-pressed="false">{zoom}</button></span><span class="no-zoom" hidden>{no_zoom}</span>
          <button type="button" class="back">{picture}</button>
          <label class="chain"><input type="checkbox" checked> {chain}</label>
        </div>
        <div class="video-box">
          <video loop playsinline preload="none" aria-label="{player}"></video>
          <p class="toast" aria-hidden="true" hidden></p>
          <p class="blocked" hidden data-no-clip="{no_clip}" data-unreadable="{unreadable}">{blocked}</p>
        </div>
        <div class="band">
          <button type="button" class="icon step" data-step="-1" aria-label="{previous}" aria-keyshortcuts="ArrowLeft">‹</button>
          <span class="toggle" role="group">{yes}{no}{doubt}</span>
          <input class="species" list="species-names" autocomplete="off" placeholder="{species}" aria-keyshortcuts="{key_species}">
          <span class="grow"><button type="button" class="to-see" aria-keyshortcuts="Shift+ArrowRight">{to_see}</button><button type="button" class="icon step" data-step="1" aria-label="{next}" aria-keyshortcuts="ArrowRight">›</button></span>
        </div>
        <p class="sr status" role="status"></p>
      </div>"#,
        view = say(Script::View),
        clip = say(Script::Clip),
        zoom = say(Script::Zoom),
        no_zoom = say(Script::NoZoom),
        picture = say(Script::Picture),
        chain = say(Script::Chain),
        player = text(&labels.label(&Label::Player(&video.name))),
        no_clip = say(Script::NoClip),
        unreadable = say(Script::Unreadable),
        blocked = say(Script::Blocked),
        previous = say(Script::PreviousTrack),
        yes = verdict("yes", "yes", Script::Yes, Script::KeyYes),
        no = verdict("no", "no", Script::No, Script::KeyNo),
        doubt = verdict("doubt", "doubt", Script::Doubt, Script::KeyDoubt),
        species = say(Script::Species),
        key_species = say(Script::KeySpecies),
        to_see = say(Script::NextToSee),
        next = say(Script::NextTrack),
    )
}
