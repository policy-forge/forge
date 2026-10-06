//! Frame the complete real held owner for private same-generation cursors.
//! Hashes are plain invalidation observations; native approval/currentness and
//! publication still require the genuine consuming factory and final fences.

use super::*;

/// Hash one borrowed framed field after each original-control chunk admission.
fn field<C: WorkControl + ?Sized>(
    shared: &SharedWork<'_, C>,
    digest: &mut Sha256,
    value: &[u8],
) -> WorkResult<()> {
    shared.charge(1)?;
    for chunk in value.chunks(CHUNK) {
        shared.charge(1)?;
        digest.update(chunk);
    }
    Ok(())
}

/// Frame complete exact private text, with length preceding every variable field.
fn text<C: WorkControl + ?Sized>(
    shared: &SharedWork<'_, C>,
    digest: &mut Sha256,
    value: &str,
) -> WorkResult<()> {
    field(shared, digest, &(value.len() as u64).to_le_bytes())?;
    field(shared, digest, value.as_bytes())
}

impl<C: WorkControl + ?Sized> HeldInputsV2<'_, C> {
    /// Derive an admitted complete physical generation from this actual held owner.
    /// Configuration, hidden/absent originals and unused directories all contribute.
    /// The result cannot assert that paths are still current without `verify_complete`.
    pub(in super::super) fn generation_digest(&self) -> WorkResult<AdmittedValue<String>> {
        let mut admission = self.admission();
        admission.retain(64, |admission| {
            let result = (|| {
                let shared = &self.builder.shared;
                let mut digest = Sha256::new();
                field(shared, &mut digest, b"forge.mcp-held-physical-generation/2\0")?;
                field(
                    shared,
                    &mut digest,
                    &[match self.builder.purpose {
                        CapturePurpose::ServerRead => 0,
                        CapturePurpose::OfflineBuild => 1,
                    }],
                )?;
                field(shared, &mut digest, &(self.builder.roots.len() as u64).to_le_bytes())?;
                for root in &self.builder.roots {
                    fresh::generation_v2::root(root.native(), &mut |bytes| {
                        field(shared, &mut digest, bytes)
                    })?;
                }
                field(shared, &mut digest, &(self.builder.files.len() as u64).to_le_bytes())?;
                for file in &self.builder.files {
                    field(
                        shared,
                        &mut digest,
                        &[match file.slot {
                            RootSlot::Project => 0,
                            RootSlot::External => 1,
                        }],
                    )?;
                    match &file.key {
                        FileKey::Config(configuration) => field(
                            shared,
                            &mut digest,
                            &[
                                0,
                                match configuration {
                                    Configuration::Discovery => 0,
                                    Configuration::Visibility => 1,
                                    Configuration::Purpose => 2,
                                },
                            ],
                        )?,
                        FileKey::Declared(key) => {
                            field(shared, &mut digest, &[1])?;
                            text(shared, &mut digest, key)?;
                        }
                    }
                    let Some(LocalState::Read(original)) = file.native.as_ref() else {
                        return Err(invalid());
                    };
                    fresh::generation_v2::original(original, &mut |bytes| {
                        field(shared, &mut digest, bytes)
                    })?;
                    match original {
                        CapturedLocal::Present(bytes, _) => {
                            field(shared, &mut digest, &[1])?;
                            field(shared, &mut digest, &(bytes.len() as u64).to_le_bytes())?;
                            let raw_hash = file.hash.as_deref().ok_or_else(invalid)?;
                            text(shared, &mut digest, raw_hash)?;
                        }
                        CapturedLocal::Absent(_) => field(shared, &mut digest, &[0])?,
                    }
                }
                field(shared, &mut digest, &(self.builder.directories.len() as u64).to_le_bytes())?;
                for directory in &self.builder.directories {
                    let native = directory.native.as_ref().ok_or_else(invalid)?;
                    fresh::generation_v2::directory(native, &mut |bytes| {
                        field(shared, &mut digest, bytes)
                    })?;
                }
                shared.charge(1)?;
                Ok(crate::hashing::lower_hex(&digest.finalize()))
            })();
            admission.phase(result, Stage::PrepareDomain)
        })
    }
}
