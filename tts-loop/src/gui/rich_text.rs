pub(crate) enum Color {
    Blue,
    Green,
    Orange,
    Red,
}

impl Color {
    fn to_str(&self) -> &'static str {
        match *self {
            Color::Green => "green",
            Color::Blue => "blue",
            Color::Red => "red",
            Color::Orange => "orange",
        }
    }
}

enum FormatInner {
    Color(Color, Box<Format>),
    Bold(Box<Format>),
    Text(String),
}
pub(crate) struct Format {
    inner: FormatInner,
}

impl Format {
    pub(crate) fn into_string(self) -> String {
        let s = String::new();
        self.into_string_impl(s)
    }

    pub(crate) fn color(c: Color, f: Box<Format>) -> Box<Format> {
        Box::new(Format {
            inner: FormatInner::Color(c, f),
        })
    }

    pub(crate) fn bold(f: Box<Format>) -> Box<Format> {
        Box::new(Format {
            inner: FormatInner::Bold(f),
        })
    }

    pub(crate) fn text(s: &str) -> Box<Format> {
        let s = v_htmlescape::escape(s).to_string();
        Box::new(Format {
            inner: FormatInner::Text(s),
        })
    }

    fn into_string_impl(self, mut s: String) -> String {
        match self.inner {
            FormatInner::Color(color, format) => {
                s.push_str("<span style=\"color:");
                s.push_str(color.to_str());
                s.push_str("\">");
                s = format.into_string_impl(s);
                s.push_str("</span>")
            }
            FormatInner::Bold(format) => {
                s.push_str("<b>");
                s = format.into_string_impl(s);
                s.push_str("</b>")
            }
            FormatInner::Text(text) => {
                s.push_str(&text);
            }
        }

        s
    }
}
