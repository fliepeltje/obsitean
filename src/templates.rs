use askama::Template;

#[derive(Template)]
#[template(path = "wiki/page.jinja2")]
pub struct WikiPage {
    pub site: crate::site::Site,
    pub note: crate::obsidian::Note,
}

impl WikiPage {
    pub fn nav(&self) -> Vec<(String, String)> {
        self.site.site_notes.iter()
            .map(|note| (note.slug.clone(), note.title.clone()))
            .collect()
    }

    pub fn markdown(&self) -> String {
        let mut md = self.note.content.clone();
        for (embed_ref, note) in &self.site.embedded_notes {
            md = md.replace(embed_ref, &note.content);
        };
        for (link_ref, note) in &self.site.linked_notes {
            md = md.replace(link_ref, &format!("[{}]({})", note.title, note.slug));
        };
        md
    }
}