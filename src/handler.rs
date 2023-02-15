use std::{path::Path, process::Command};

use crate::App;

pub async fn  handle_enter(
    app: &App,
    zotero_storage_dir: &Path,
    viewer: &str,
) -> anyhow::Result<()> {
    match app.filtered_documents.state.selected() {
        Some(index) => {
            let doc = app.filtered_documents.items.get(index).unwrap();
            match &doc.attachments {
                Some(attachments) => {
                    let attachment = &doc.attachments.as_ref().unwrap()[0];
                    let path = zotero_storage_dir
                        .join(&attachment.key.as_ref().unwrap())
                        .join(&attachment.path.as_ref().unwrap()[":storage".len()..]);
                    Command::new("zathura").arg(&path).spawn()?;
                }
                None => {}
            }
        }
        None => {}
    }
    Ok(())
}
