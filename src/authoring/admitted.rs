//! Plain pre-growth admission over the caller's unchanged work owner.
//!
//! Fixed-size serialization metrics retain no Value, string, encoded buffer or plan.
//! These logical descriptors are not allocator measurements or native source authority.

use super::borrowed::{AuthoringCharge, BorrowedAuthoringError};
use crate::ForgeError;
use crate::workspace::preparation::{ProgressUpdate, Stage, WorkControl, WorkError};
use serde::Serialize;
use serde::ser::{
    self, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use std::fmt;

/// One borrowed caller control and the actual admission callback; no renewed allowance.
pub(super) struct Admission<'a, E, F: FnMut(AuthoringCharge) -> Result<(), E>> {
    pub(super) control: &'a mut dyn WorkControl,
    pub(super) admit: &'a mut F,
    /// Type-only callback error marker; no owned error, budget or runtime authority exists.
    pub(super) _error: std::marker::PhantomData<fn() -> E>,
}
impl<E, F: FnMut(AuthoringCharge) -> Result<(), E>> Admission<'_, E, F> {
    /// Preserve callback admission first, then the actual original cooperative stop.
    pub(super) fn step(&mut self) -> Result<(), BorrowedAuthoringError<E>> {
        (self.admit)(AuthoringCharge::Checkpoint).map_err(BorrowedAuthoringError::Admission)?;
        self.control
            .checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
            .map_err(BorrowedAuthoringError::Work)?;
        if let Some(reason) = self.control.interruption() {
            return Err(BorrowedAuthoringError::Work(WorkError::Interrupted(reason)));
        }
        Ok(())
    }
    /// Charge the complete repeated primitive work before it is performed.
    pub(super) fn work(
        &mut self,
        visits: usize,
        bytes: usize,
        matching: usize,
    ) -> Result<(), BorrowedAuthoringError<E>> {
        (self.admit)(AuthoringCharge::Work { visits, byte_work: bytes, matching_steps: matching })
            .map_err(BorrowedAuthoringError::Admission)
    }
    /// Admit the complete logical retained/temporary payload before allocation.
    pub(super) fn reserve(&mut self, bytes: usize) -> Result<(), BorrowedAuthoringError<E>> {
        (self.admit)(AuthoringCharge::Reserve { logical_bytes: bytes })
            .map_err(BorrowedAuthoringError::Admission)
    }
    /// Notify the original caller about Capacity even if its callback accepts that notice.
    pub(super) fn capacity<T>(&mut self) -> Result<T, BorrowedAuthoringError<E>> {
        match (self.admit)(AuthoringCharge::Capacity) {
            Err(error) => Err(BorrowedAuthoringError::Admission(error)),
            Ok(()) => Err(BorrowedAuthoringError::Capacity),
        }
    }
    /// Checked complete sums never become a successful truncated descriptor.
    pub(super) fn add(&mut self, a: usize, b: usize) -> Result<usize, BorrowedAuthoringError<E>> {
        a.checked_add(b).map_or_else(|| self.capacity(), Ok)
    }
    /// Checked complete Cartesian/byte products preserve the same first stop.
    pub(super) fn mul(&mut self, a: usize, b: usize) -> Result<usize, BorrowedAuthoringError<E>> {
        a.checked_mul(b).map_or_else(|| self.capacity(), Ok)
    }
    /// Native success and ordinary Domain failure both cross the original post-phase fence.
    pub(super) fn native<T>(
        &mut self,
        work: impl FnOnce() -> Result<T, ForgeError>,
    ) -> Result<T, BorrowedAuthoringError<E>> {
        self.step()?;
        let result = work();
        self.step()?;
        result.map_err(BorrowedAuthoringError::Domain)
    }
    /// An observed Admission/Work/Capacity is first; ordinary results require a final fence.
    pub(super) fn finish<T>(
        &mut self,
        result: Result<T, BorrowedAuthoringError<E>>,
    ) -> Result<T, BorrowedAuthoringError<E>> {
        match result {
            Err(
                error @ (BorrowedAuthoringError::Admission(_)
                | BorrowedAuthoringError::Work(_)
                | BorrowedAuthoringError::Capacity),
            ) => Err(error),
            ordinary => {
                self.step()?;
                ordinary
            }
        }
    }
    /// Gate both the inspection and every later complete comparison of these actual operands.
    pub(super) fn equal(&mut self, a: &str, b: &str) -> Result<bool, BorrowedAuthoringError<E>> {
        self.step()?;
        let bytes = self.add(a.len(), b.len())?;
        self.work(1, bytes, 1)?;
        Ok(a == b)
    }
    /// Inspect already held typed fields using a fixed-size, admission-bounded serializer.
    pub(super) fn measure<T: Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<Metrics, BorrowedAuthoringError<E>> {
        let mut meter = Meter { admission: self, metrics: Metrics::default(), depth: 0 };
        value.serialize(&mut meter).map_err(|error| error.0)?;
        Ok(meter.metrics)
    }
    /// Before strict Value/typed decoding, count lexical slots without accepting JSON syntax.
    pub(super) fn raw(
        &mut self,
        bytes: &[u8],
        forms: usize,
    ) -> Result<(), BorrowedAuthoringError<E>> {
        self.step()?;
        self.work(0, bytes.len(), 0)?;
        let mut quoted = false;
        let mut escaped = false;
        let mut primitive = false;
        let mut slots = 0usize;
        for chunk in bytes.chunks(32 * 1024) {
            self.step()?;
            for &byte in chunk {
                if quoted {
                    if escaped {
                        escaped = false;
                    } else if byte == b'\\' {
                        escaped = true;
                    } else if byte == b'"' {
                        quoted = false;
                    }
                    continue;
                }
                match byte {
                    b'"' => {
                        slots = self.add(slots, 1)?;
                        quoted = true;
                        primitive = false;
                    }
                    b'{' | b'[' => {
                        slots = self.add(slots, 1)?;
                        primitive = false;
                    }
                    b'}' | b']' | b',' | b':' | b' ' | b'\r' | b'\n' | b'\t' => primitive = false,
                    _ if !primitive => {
                        slots = self.add(slots, 1)?;
                        primitive = true;
                    }
                    _ => {}
                }
            }
        }
        // One128-byte quantum per Value/key/typed slot and exact raw string upper bound.
        // Value parse/null walk/typed conversion are separate from later semantic phases.
        let headers = self.mul(slots, 128)?;
        let payload = self.add(headers, bytes.len())?;
        let payload = self.add(payload, 512)?;
        let payload = self.mul(payload, forms)?;
        self.reserve(payload)?;
        let passes = self.add(forms, 2)?;
        let work = self.mul(bytes.len(), passes)?;
        let visits = self.mul(slots, passes)?;
        self.work(visits, work, 0)
    }
    /// Bound complete native JSON materialization, canonical sorting and equality operands.
    pub(super) fn materialize(
        &mut self,
        metric: Metrics,
        copies: usize,
        passes: usize,
    ) -> Result<(), BorrowedAuthoringError<E>> {
        let owned = self.mul(metric.logical, copies)?;
        let encoded = self.mul(metric.encoded, copies)?;
        let total = self.add(owned, encoded)?;
        self.reserve(total)?;
        let bytes = self.mul(metric.encoded, passes)?;
        let visits = self.mul(metric.nodes, passes)?;
        // Full cloned/serialized operands are linear here. Registry/canonical phases
        // separately admit their actual complete key sets before sorting or lookup.
        self.work(visits, bytes, 0)
    }
}

/// Fixed-size complete serialization descriptor; every numeric field uses checked arithmetic.
#[derive(Clone, Copy, Default)]
pub(super) struct Metrics {
    pub(super) nodes: usize,
    pub(super) text: usize,
    pub(super) logical: usize,
    pub(super) encoded: usize,
}
/// Private serializer error retains actual admission/work causes, never emits input prose.
struct MeasureError<E>(BorrowedAuthoringError<E>);
impl<E> fmt::Debug for MeasureError<E> {
    /// Emit only the fixed metric diagnostic label, with no private native prose.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("authoring metric error")
    }
}
impl<E> fmt::Display for MeasureError<E> {
    /// Emit only the fixed metric diagnostic label, with no private native prose.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("authoring metric error")
    }
}
impl<E> std::error::Error for MeasureError<E> {}
impl<E> ser::Error for MeasureError<E> {
    /// Refuse unsupported serde metric output using an ordinary fixed native error.
    fn custom<T: fmt::Display>(_message: T) -> Self {
        Self(BorrowedAuthoringError::Domain(super::error("cannot measure authoring evidence")))
    }
}
/// Stack-only meter, never a serialized plan or caller-supplied completeness flag.
struct Meter<'a, 'b, E, F: FnMut(AuthoringCharge) -> Result<(), E>> {
    admission: &'a mut Admission<'b, E, F>,
    metrics: Metrics,
    depth: usize,
}
impl<E, F: FnMut(AuthoringCharge) -> Result<(), E>> Meter<'_, '_, E, F> {
    /// Account before reading or counting a complete already-held primitive operand.
    fn node(&mut self, text: usize, encoded: usize) -> Result<(), MeasureError<E>> {
        self.admission.step().map_err(MeasureError)?;
        self.admission.work(1, text, 0).map_err(MeasureError)?;
        self.metrics.nodes = self.admission.add(self.metrics.nodes, 1).map_err(MeasureError)?;
        self.metrics.text = self.admission.add(self.metrics.text, text).map_err(MeasureError)?;
        let logical = self.admission.add(128, text).map_err(MeasureError)?;
        self.metrics.logical =
            self.admission.add(self.metrics.logical, logical).map_err(MeasureError)?;
        self.metrics.encoded =
            self.admission.add(self.metrics.encoded, encoded).map_err(MeasureError)?;
        Ok(())
    }
    /// Six encoded bytes per UTF8 source byte plus32 syntax bytes cover JSON escaping/punctuation.
    fn string(&mut self, value: &str) -> Result<(), MeasureError<E>> {
        let encoded = self.admission.mul(value.len(), 6).map_err(MeasureError)?;
        let encoded = self.admission.add(encoded, 32).map_err(MeasureError)?;
        self.node(value.len(), encoded)
    }
    /// Bound serializer recursion before entering a compound child, with no heap stack.
    fn begin(&mut self) -> Result<(), MeasureError<E>> {
        if self.depth >= 128 {
            return self.admission.capacity().map_err(MeasureError);
        }
        self.node(0, 32)?;
        self.depth += 1;
        Ok(())
    }
}
/// The only compound state is a stack borrow of the same meter.
struct Compound<'a, 'b, 'c, E, F: FnMut(AuthoringCharge) -> Result<(), E>> {
    meter: &'a mut Meter<'b, 'c, E, F>,
}
impl<E, F: FnMut(AuthoringCharge) -> Result<(), E>> Compound<'_, '_, '_, E, F> {
    /// Visit one complete already-held child through the same stack meter.
    fn value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), MeasureError<E>> {
        value.serialize(&mut *self.meter)
    }
    /// Measure the complete actual key and value before entering the next field.
    fn field<T: Serialize + ?Sized>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), MeasureError<E>> {
        self.meter.string(key)?;
        self.value(value)
    }
    /// Finish the current compound stack depth without retaining a container.
    fn done(self) {
        self.meter.depth -= 1;
    }
}
impl<'a, 'b, 'c, E, F: FnMut(AuthoringCharge) -> Result<(), E>> ser::Serializer
    for &'a mut Meter<'b, 'c, E, F>
{
    type Ok = ();
    type Error = MeasureError<E>;
    type SerializeSeq = Compound<'a, 'b, 'c, E, F>;
    type SerializeTuple = Compound<'a, 'b, 'c, E, F>;
    type SerializeTupleStruct = Compound<'a, 'b, 'c, E, F>;
    type SerializeTupleVariant = Compound<'a, 'b, 'c, E, F>;
    type SerializeMap = Compound<'a, 'b, 'c, E, F>;
    type SerializeStruct = Compound<'a, 'b, 'c, E, F>;
    type SerializeStructVariant = Compound<'a, 'b, 'c, E, F>;
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_bool(self, _v: bool) -> Result<(), Self::Error> {
        self.node(0, 32)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_i8(self, v: i8) -> Result<(), Self::Error> {
        self.serialize_i64(i64::from(v))
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_i16(self, v: i16) -> Result<(), Self::Error> {
        self.serialize_i64(i64::from(v))
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_i32(self, v: i32) -> Result<(), Self::Error> {
        self.serialize_i64(i64::from(v))
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_i64(self, _v: i64) -> Result<(), Self::Error> {
        self.node(0, 32)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_u8(self, v: u8) -> Result<(), Self::Error> {
        self.serialize_u64(u64::from(v))
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_u16(self, v: u16) -> Result<(), Self::Error> {
        self.serialize_u64(u64::from(v))
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_u32(self, v: u32) -> Result<(), Self::Error> {
        self.serialize_u64(u64::from(v))
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_u64(self, _v: u64) -> Result<(), Self::Error> {
        self.node(0, 32)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_i128(self, _v: i128) -> Result<(), Self::Error> {
        self.node(0, 64)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_u128(self, _v: u128) -> Result<(), Self::Error> {
        self.node(0, 64)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_f32(self, v: f32) -> Result<(), Self::Error> {
        self.serialize_f64(f64::from(v))
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_f64(self, _v: f64) -> Result<(), Self::Error> {
        self.node(0, 1024)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_char(self, v: char) -> Result<(), Self::Error> {
        self.serialize_str(v.encode_utf8(&mut [0; 4]))
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_str(self, v: &str) -> Result<(), Self::Error> {
        self.string(v)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_bytes(self, v: &[u8]) -> Result<(), Self::Error> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            SerializeSeq::serialize_element(&mut seq, byte)?;
        }
        SerializeSeq::end(seq)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_none(self) -> Result<(), Self::Error> {
        self.node(0, 32)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<(), Self::Error> {
        v.serialize(self)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_unit(self) -> Result<(), Self::Error> {
        self.node(0, 32)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Self::Error> {
        self.serialize_unit()
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
    ) -> Result<(), Self::Error> {
        self.serialize_str(variant)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        v.serialize(self)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        self.string(variant)?;
        self.begin()?;
        v.serialize(&mut *self)?;
        self.depth -= 1;
        Ok(())
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.begin()?;
        Ok(Compound { meter: self })
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.string(variant)?;
        self.serialize_seq(Some(len))
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.serialize_seq(len)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.string(variant)?;
        self.serialize_seq(Some(len))
    }
    /// Refuse a Display-only field before any unadmitted formatting buffer.
    fn collect_str<T: fmt::Display + ?Sized>(self, _value: &T) -> Result<(), Self::Error> {
        // Native authoring/report contracts serialize exact strings directly. Never
        // allocate an unadmitted Display buffer for an unsupported future field.
        Err(ser::Error::custom("unsupported authoring Display field"))
    }
}
impl<E, F: FnMut(AuthoringCharge) -> Result<(), E>> SerializeSeq for Compound<'_, '_, '_, E, F> {
    type Ok = ();
    type Error = MeasureError<E>;
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Self::Error> {
        self.value(v)
    }
    /// Finish the current compound stack depth without retaining a container.
    fn end(self) -> Result<(), Self::Error> {
        self.done();
        Ok(())
    }
}
impl<E, F: FnMut(AuthoringCharge) -> Result<(), E>> SerializeTuple for Compound<'_, '_, '_, E, F> {
    type Ok = ();
    type Error = MeasureError<E>;
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Self::Error> {
        self.value(v)
    }
    /// Finish the current compound stack depth without retaining a container.
    fn end(self) -> Result<(), Self::Error> {
        self.done();
        Ok(())
    }
}
impl<E, F: FnMut(AuthoringCharge) -> Result<(), E>> SerializeTupleStruct
    for Compound<'_, '_, '_, E, F>
{
    type Ok = ();
    type Error = MeasureError<E>;
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_field<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Self::Error> {
        self.value(v)
    }
    /// Finish the current compound stack depth without retaining a container.
    fn end(self) -> Result<(), Self::Error> {
        self.done();
        Ok(())
    }
}
impl<E, F: FnMut(AuthoringCharge) -> Result<(), E>> SerializeTupleVariant
    for Compound<'_, '_, '_, E, F>
{
    type Ok = ();
    type Error = MeasureError<E>;
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_field<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Self::Error> {
        self.value(v)
    }
    /// Finish the current compound stack depth without retaining a container.
    fn end(self) -> Result<(), Self::Error> {
        self.done();
        Ok(())
    }
}
impl<E, F: FnMut(AuthoringCharge) -> Result<(), E>> SerializeMap for Compound<'_, '_, '_, E, F> {
    type Ok = ();
    type Error = MeasureError<E>;
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_key<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Self::Error> {
        self.value(v)
    }
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Self::Error> {
        self.value(v)
    }
    /// Finish the current compound stack depth without retaining a container.
    fn end(self) -> Result<(), Self::Error> {
        self.done();
        Ok(())
    }
}
impl<E, F: FnMut(AuthoringCharge) -> Result<(), E>> SerializeStruct for Compound<'_, '_, '_, E, F> {
    type Ok = ();
    type Error = MeasureError<E>;
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        k: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        self.field(k, v)
    }
    /// Finish the current compound stack depth without retaining a container.
    fn end(self) -> Result<(), Self::Error> {
        self.done();
        Ok(())
    }
}
impl<E, F: FnMut(AuthoringCharge) -> Result<(), E>> SerializeStructVariant
    for Compound<'_, '_, '_, E, F>
{
    type Ok = ();
    type Error = MeasureError<E>;
    /// Measure this complete serde operand using the same stack-only admission meter.
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        k: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        self.field(k, v)
    }
    /// Finish the current compound stack depth without retaining a container.
    fn end(self) -> Result<(), Self::Error> {
        self.done();
        Ok(())
    }
}
