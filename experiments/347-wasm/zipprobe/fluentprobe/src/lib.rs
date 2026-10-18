use fluent_bundle::{FluentBundle, FluentResource};
use unic_langid::langid;

#[unsafe(no_mangle)]
pub extern "C" fn probe() -> usize {
    let resource = FluentResource::try_new("hello = Bonjour { $name }\n".to_owned()).map_or(0, |_| 1);
    let bundle: FluentBundle<FluentResource> = FluentBundle::new(vec![langid!("fr")]);
    let now = jiff::Zoned::now().strftime("%Y-%m-%d").to_string();
    resource + bundle.locales.len() + now.len()
}
