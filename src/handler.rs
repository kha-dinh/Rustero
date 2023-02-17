use std::{path::Path, process::Command};

use crate::App;

// TODO: Should use user_config instead of viewer
pub async fn handle_enter(
    app: &App,
    zotero_storage_dir: &Path,
    viewer: &str,
) -> anyhow::Result<()> {
    match app.filtered_documents.state.selected() {
        Some(index) => {
            let doc = app.filtered_documents.items.get(index).unwrap();
            match &doc.borrow().attachments {
                Some(attachments) => {
                    for att in attachments {
                        // TODO: Choose between different attachments
                        if att.contentType.as_ref().unwrap().contains("pdf") {
                            let path = zotero_storage_dir
                                .join(&att.key.as_ref().unwrap())
                                .join(&att.path.as_ref().unwrap()[":storage".len()..]);
                            Command::new(viewer).arg(&path).spawn()?;
                            break;
                        }
                    }
                    // let attachment = &attachments[0];
                }
                None => {}
            }
        }
        None => {}
    }
    Ok(())
}
