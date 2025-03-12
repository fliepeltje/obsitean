use askama::Template;

#[derive(Template)]
#[template(path = "wiki/page.jinja2")]
pub struct WikiPage {
    pub site: crate::site::Site,
    pub note: crate::obsidian::Note,
}

impl WikiPage {
    pub fn nav(&self) -> Vec<(String, String)> {
        let mut nav: Vec<(String, String)> = self.site.site_notes.iter()
            .map(|note| (note.slug.clone(), note.title.clone()))
            .collect();
        
        // Sort by alphabetical order
        nav.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
        
        // Move "index" to the front if it exists
        if let Some(index_pos) = nav.iter().position(|(slug, _)| slug == "index") {
            let index_item = nav.remove(index_pos);
            nav.insert(0, index_item);
        }
        
        nav
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