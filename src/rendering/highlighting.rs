use syntect::dumps::from_binary;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

lazy_static! {
    pub static ref THEME_SET: ThemeSet = from_binary(include_bytes!("../../sublime_themes/all.themedump"));
}

thread_local! {
    pub static SYNTAX_SET: SyntaxSet = {
        let mut syntax_set: SyntaxSet = from_binary(include_bytes!("../../sublime_syntaxes/newlines.packdump"));
        syntax_set.link_syntaxes();
        syntax_set
    }
}
