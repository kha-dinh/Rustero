use std::{path::Path, process::Command};

use crate::App;

// TODO: Should use user_config instead of viewer
pub async fn handle_enter(
    app: &App,
    zotero_storage_dir: &Path,
    viewer: &str,
) -> anyhow::Result<()> {
    if let Some(selected) = app.get_selected_doc() {
        match &selected.borrow().attachments {
            Some(attachments) => {
                for att in &attachments.items {
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
    };
    Ok(())
}
