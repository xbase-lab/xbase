use crate::{Error, Result};
use std::path::Path;

pub trait PathExt {
    fn name(&self) -> Option<String>;
    fn unique_name(&self) -> Option<String>;
    fn abbrv(&self) -> Result<&Path>;
}

impl PathExt for Path {
    fn name(&self) -> Option<String> {
        let mut name = self.file_name().and_then(|os| os.to_str())?.to_string();

        let name = name.remove(0).to_uppercase().to_string() + &name;

        Some(name)
    }

    fn unique_name(&self) -> Option<String> {
        Some(
            self.strip_prefix(self.ancestors().nth(3)?)
                .ok()?
                .display()
                .to_string()
                .replace("/", "_"),
        )
    }

    fn abbrv(&self) -> Result<&Path> {
        let ancestors = self.ancestors().nth(3);
        let ancestors =
            ancestors.ok_or_else(|| Error::Unexpected("Getting 3 parent of a path".into()))?;
        self.strip_prefix(ancestors)
            .map_err(|e| Error::Unexpected(e.to_string()))
    }
}
