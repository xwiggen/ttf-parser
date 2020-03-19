// https://docs.microsoft.com/en-us/typography/opentype/spec/mvar

use crate::{Tag, NormalizedCoord};
use crate::parser::{Stream, Offset, Offset16, LazyArray16};
use crate::var_store::ItemVariationStore;
use crate::raw::mvar as raw;

#[derive(Clone, Copy)]
pub(crate) struct Table<'a> {
    variation_store: ItemVariationStore<'a>,
    records: LazyArray16<'a, raw::ValueRecord>,
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let version: u32 = s.read()?;
        if version != 0x00010000 {
            return None;
        }

        s.skip::<u16>(); // reserved
        let value_record_size: u16 = s.read()?;

        if usize::from(value_record_size) != raw::ValueRecord::SIZE {
            return None;
        }

        let count: u16 = s.read()?;
        if count == 0 {
            return None;
        }

        let var_store_offset = s.read::<Option<Offset16>>()??.to_usize();
        let records = s.read_array16::<raw::ValueRecord>(count)?;
        let variation_store = ItemVariationStore::parse(Stream::new_at(data, var_store_offset))?;

        Some(Table {
            variation_store,
            records,
        })
    }

    pub fn metrics_offset(&self, tag: Tag, coordinates: &[NormalizedCoord]) -> Option<f32> {
        let (_, record) = self.records.binary_search_by(|r| r.value_tag().cmp(&tag))?;
        self.variation_store.parse_delta(
            record.delta_set_outer_index(),
            record.delta_set_inner_index(),
            coordinates
        )
    }
}
