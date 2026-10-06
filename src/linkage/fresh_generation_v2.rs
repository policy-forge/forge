//! Borrow only genuine native generation fields for private cursor invalidation.
//! This stream allocates no path copies, performs no IO and creates no proof.
//! The consuming owner must admit every supplied chunk before hashing it and
//! retain its actual native owners through the complete final input fence.

use super::{CapturedLocal, DirectoryGeneration, HeldDirectoryGeneration, RootGeneration};

/// Frame the exact native byte spelling without lossy display conversion.
fn path<E>(
    value: &std::path::Path,
    emit: &mut impl FnMut(&[u8]) -> Result<(), E>,
) -> Result<(), E> {
    let bytes = value.as_os_str().as_encoded_bytes();
    emit(&(bytes.len() as u64).to_le_bytes())?;
    emit(bytes)
}

/// Frame every actual directory observation in its original traversal order.
fn ancestors<E>(
    values: &[DirectoryGeneration],
    emit: &mut impl FnMut(&[u8]) -> Result<(), E>,
) -> Result<(), E> {
    emit(&(values.len() as u64).to_le_bytes())?;
    for value in values {
        path(&value.path, emit)?;
        emit(&value.identity.0.to_le_bytes())?;
        emit(&value.identity.1.to_le_bytes())?;
    }
    Ok(())
}

/// Stream the complete actual qualified root and every held root ancestor.
pub(crate) fn root<E>(
    value: &RootGeneration,
    emit: &mut impl FnMut(&[u8]) -> Result<(), E>,
) -> Result<(), E> {
    emit(b"root\0")?;
    path(&value.path, emit)?;
    emit(&value.identity.0.to_le_bytes())?;
    emit(&value.identity.1.to_le_bytes())?;
    ancestors(&value.ancestors, emit)
}

/// Stream the real present instance or typed missing component and full ancestry.
/// Complete original bytes are separately hashed by the owning capture reader.
pub(crate) fn original<E>(
    value: &CapturedLocal,
    emit: &mut impl FnMut(&[u8]) -> Result<(), E>,
) -> Result<(), E> {
    emit(b"original\0")?;
    match value {
        CapturedLocal::Present(_, file) => {
            emit(&[1])?;
            emit(&file.identity.0.to_le_bytes())?;
            emit(&file.identity.1.to_le_bytes())?;
            root(&file.root, emit)?;
            path(&file.relative, emit)?;
            ancestors(&file.ancestors, emit)
        }
        CapturedLocal::Absent(absence) => {
            emit(&[0])?;
            root(&absence.root, emit)?;
            path(&absence.relative, emit)?;
            path(&absence.missing, emit)?;
            ancestors(&absence.ancestors, emit)
        }
    }
}

/// Stream the complete genuine directory holder, including unused declarations.
pub(crate) fn directory<E>(
    value: &HeldDirectoryGeneration,
    emit: &mut impl FnMut(&[u8]) -> Result<(), E>,
) -> Result<(), E> {
    emit(b"directory\0")?;
    root(&value.root, emit)?;
    path(&value.relative, emit)?;
    emit(&value.identity.0.to_le_bytes())?;
    emit(&value.identity.1.to_le_bytes())?;
    ancestors(&value.ancestors, emit)
}
