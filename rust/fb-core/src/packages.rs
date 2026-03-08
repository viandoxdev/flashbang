use typst::{diag::FileResult, foundations::Bytes, syntax::{FileId, Source}};

use crate::world::WorldState;

pub trait PackageProvider: Send + Sync + 'static {
    fn get_package_source(&self, id: FileId, world: &WorldState) -> FileResult<Source>;
    fn get_package_file(&self, id: FileId, world: &WorldState) -> FileResult<Bytes>;
}
