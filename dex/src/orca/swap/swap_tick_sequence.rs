#![allow(clippy::doc_overindented_list_items)]
#![allow(clippy::empty_line_after_doc_comments)]

use crate::orca::{
    swap::{
        tick::{MAX_TICK_INDEX, MIN_TICK_INDEX},
        TICK_ARRAY_SIZE,
    },
    OrcaTick, OrcaTickArray,
};

type ProxiedTickArray = OrcaTickArray;
type Tick = OrcaTick;

pub struct SwapTickSequence<'a> {
    pub arrays: Vec<&'a ProxiedTickArray>,
}

fn get_offset(tick_index: i32, start_tick_index: i32, tick_spacing: u16) -> isize {
    // TODO: replace with i32.div_floor once not experimental
    let lhs = tick_index - start_tick_index;
    // rhs(tick_spacing) is always positive number (non zero)
    let rhs = tick_spacing as i32;
    let d = lhs / rhs;
    let r = lhs % rhs;
    let o = if r < 0 { d - 1 } else { d };
    o as isize
}

impl OrcaTickArray {
    pub fn start_tick_index(&self) -> i32 {
        self.start_tick_index
    }

    /// Checks that this array holds the next tick index for the current tick index, given the pool's tick spacing & search direction.
    ///
    /// unshifted checks on [start, start + TICK_ARRAY_SIZE * tick_spacing)
    /// shifted checks on [start - tick_spacing, start + (TICK_ARRAY_SIZE - 1) * tick_spacing) (adjusting range by -tick_spacing)
    ///
    /// shifted == !a_to_b
    ///
    /// For a_to_b swaps, price moves left. All searchable ticks in this tick-array's range will end up in this tick's usable ticks.
    /// The search range is therefore the range of the tick-array.
    ///
    /// For b_to_a swaps, this tick-array's left-most ticks can be the 'next' usable tick-index of the previous tick-array.
    /// The right-most ticks also points towards the next tick-array. The search range is therefore shifted by 1 tick-spacing.
    fn in_search_range(&self, tick_index: i32, tick_spacing: u16, shifted: bool) -> bool {
        let mut lower = self.start_tick_index();
        let mut upper = self.start_tick_index() + TICK_ARRAY_SIZE * tick_spacing as i32;
        if shifted {
            lower -= tick_spacing as i32;
            upper -= tick_spacing as i32;
        }
        tick_index >= lower && tick_index < upper
    }

    fn check_in_array_bounds(&self, tick_index: i32, tick_spacing: u16) -> bool {
        self.in_search_range(tick_index, tick_spacing, false)
    }

    fn tick_offset(&self, tick_index: i32, tick_spacing: u16) -> anyhow::Result<isize> {
        if tick_spacing == 0 {
            return Err(anyhow::format_err!("InvalidTickSpacing"));
        }

        Ok(get_offset(tick_index, self.start_tick_index(), tick_spacing))
    }

    /// Get the Tick object at the given tick-index & tick-spacing
    ///
    /// # Parameters
    /// - `tick_index` - the tick index the desired Tick object is stored in
    /// - `tick_spacing` - A u8 integer of the tick spacing for this whirlpool
    ///
    /// # Returns
    /// - `&Tick`: A reference to the desired Tick object
    /// - `TickNotFound`: - The provided tick-index is not an initializable tick index in this Whirlpool w/ this tick-spacing.
    fn get_tick(&self, tick_index: i32, tick_spacing: u16) -> anyhow::Result<Tick> {
        if !self.check_in_array_bounds(tick_index, tick_spacing)
            || !Tick::check_is_usable_tick(tick_index, tick_spacing)
        {
            return Err(anyhow::format_err!("TickNotFound"));
        }
        let offset = self.tick_offset(tick_index, tick_spacing)?;
        if offset < 0 {
            return Err(anyhow::format_err!("TickNotFound"));
        }
        Ok(self.ticks[offset as usize])
    }

    /// Search for the next initialized tick in this array.
    ///
    /// # Parameters
    /// - `tick_index` - A i32 integer representing the tick index to start searching for
    /// - `tick_spacing` - A u8 integer of the tick spacing for this whirlpool
    /// - `a_to_b` - If the trade is from a_to_b, the search will move to the left and the starting search tick is inclusive.
    ///              If the trade is from b_to_a, the search will move to the right and the starting search tick is not inclusive.
    ///
    /// # Returns
    /// - `Some(i32)`: The next initialized tick index of this array
    /// - `None`: An initialized tick index was not found in this array
    /// - `InvalidTickArraySequence` - error if `tick_index` is not a valid search tick for the array
    /// - `InvalidTickSpacing` - error if the provided tick spacing is 0
    fn get_next_init_tick_index(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> anyhow::Result<Option<i32>> {
        if !self.in_search_range(tick_index, tick_spacing, !a_to_b) {
            return Err(anyhow::format_err!("InvalidTickArraySequence"));
        }

        let mut curr_offset = match self.tick_offset(tick_index, tick_spacing) {
            Ok(value) => value as i32,
            Err(e) => return Err(e),
        };

        // For a_to_b searches, the search moves to the left. The next possible init-tick can be the 1st tick in the current offset
        // For b_to_a searches, the search moves to the right. The next possible init-tick cannot be within the current offset
        if !a_to_b {
            curr_offset += 1;
        }

        while (0..TICK_ARRAY_SIZE).contains(&curr_offset) {
            let curr_tick = self.ticks[curr_offset as usize];
            if curr_tick.initialized {
                return Ok(Some((curr_offset * tick_spacing as i32) + self.start_tick_index));
            }

            curr_offset = if a_to_b { curr_offset - 1 } else { curr_offset + 1 };
        }

        Ok(None)
    }
}

impl<'a> SwapTickSequence<'a> {
    // pub fn new(
    //     ta0: LoadedTickArrayMut<'a>,
    //     ta1: Option<LoadedTickArrayMut<'a>>,
    //     ta2: Option<LoadedTickArrayMut<'a>>,
    // ) -> Self {
    //     Self::new_with_proxy(
    //         ProxiedTickArray::new_initialized(ta0),
    //         ta1.map(ProxiedTickArray::new_initialized),
    //         ta2.map(ProxiedTickArray::new_initialized),
    //     )
    // }

    // pub(crate) fn new_with_proxy(
    //     ta0: ProxiedTickArray<'a>,
    //     ta1: Option<ProxiedTickArray<'a>>,
    //     ta2: Option<ProxiedTickArray<'a>>,
    // ) -> Self {
    //     let mut vec = Vec::with_capacity(3);
    //     vec.push(ta0);
    //     if let Some(ta1) = ta1 {
    //         vec.push(ta1);
    //     }
    //     if let Some(ta2) = ta2 {
    //         vec.push(ta2);
    //     }
    //     Self { arrays: vec }
    // }

    /// Get the Tick object at the given tick-index & tick-spacing
    ///
    /// # Parameters
    /// - `array_index` - the array index that the tick of this given tick-index would be stored in
    /// - `tick_index` - the tick index the desired Tick object is stored in
    /// - `tick_spacing` - A u8 integer of the tick spacing for this whirlpool
    ///
    /// # Returns
    /// - `&Tick`: A reference to the desired Tick object
    /// - `TickArrayIndexOutofBounds` - The provided array-index is out of bounds
    /// - `TickNotFound`: - The provided tick-index is not an initializable tick index in this Whirlpool w/ this tick-spacing.
    pub fn get_tick(&self, array_index: usize, tick_index: i32, tick_spacing: u16) -> anyhow::Result<Tick> {
        let array = self.arrays.get(array_index);
        match array {
            Some(array) => array.get_tick(tick_index, tick_spacing),
            _ => Err(anyhow::format_err!("TickArrayIndexOutofBounds")),
        }
    }

    /// Updates the Tick object at the given tick-index & tick-spacing
    ///
    /// # Parameters
    /// - `array_index` - the array index that the tick of this given tick-index would be stored in
    /// - `tick_index` - the tick index the desired Tick object is stored in
    /// - `tick_spacing` - A u8 integer of the tick spacing for this whirlpool
    /// - `update` - A reference to a TickUpdate object to update the Tick object at the given index
    ///
    /// # Errors
    /// - `TickArrayIndexOutofBounds` - The provided array-index is out of bounds
    /// - `TickNotFound`: - The provided tick-index is not an initializable tick index in this Whirlpool w/ this tick-spacing.
    // pub fn update_tick(
    //     &mut self,
    //     array_index: usize,
    //     tick_index: i32,
    //     tick_spacing: u16,
    //     update: &TickUpdate,
    // ) -> anyhow::Result<()> {
    //     let array = self.arrays.get_mut(array_index);
    //     match array {
    //         Some(array) => {
    //             array.update_tick(tick_index, tick_spacing, update)?;
    //             Ok(())
    //         }
    //         _ => Err(ErrorCode::TickArrayIndexOutofBounds.into()),
    //     }
    // }

    pub fn get_tick_offset(&self, array_index: usize, tick_index: i32, tick_spacing: u16) -> anyhow::Result<isize> {
        let array = self.arrays.get(array_index);
        match array {
            Some(array) => array.tick_offset(tick_index, tick_spacing),
            _ => Err(anyhow::format_err!("TickArrayIndexOutofBounds")),
        }
    }

    /// Get the next initialized tick in the provided tick range
    ///
    /// # Parameters
    /// - `tick_index` - the tick index to start searching from
    /// - `tick_spacing` - A u8 integer of the tick spacing for this whirlpool
    /// - `a_to_b` - If the trade is from a_to_b, the search will move to the left and the starting search tick is inclusive.
    ///              If the trade is from b_to_a, the search will move to the right and the starting search tick is not inclusive.
    /// - `start_array_index` -
    ///
    /// # Returns
    /// - `(usize, i32, &mut Tick)`: The array_index which the next initialized index was found, the next initialized tick-index & a mutable reference to that tick
    /// - `TickArraySequenceInvalidIndex` - The swap loop provided an invalid array index to query the next tick in.
    /// - `InvalidTickArraySequence`: - User provided tick-arrays are not in sequential order required to proceed in this trade direction.
    pub fn get_next_initialized_tick_index(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
        start_array_index: usize,
    ) -> anyhow::Result<(usize, i32)> {
        let ticks_in_array = TICK_ARRAY_SIZE * tick_spacing as i32;
        let mut search_index = tick_index;
        let mut array_index = start_array_index;

        // Keep looping the arrays until an initialized tick index in the subsequent tick-arrays found.
        loop {
            // If we get to the end of the array sequence and next_index is still not found, throw error
            let next_array = match self.arrays.get(array_index) {
                Some(array) => array,
                None => return Err(anyhow::format_err!("TickArraySequenceInvalidIndex")),
            };

            let next_index = next_array.get_next_init_tick_index(search_index, tick_spacing, a_to_b)?;

            match next_index {
                Some(next_index) => {
                    return Ok((array_index, next_index));
                }
                None => {
                    // If we are at the last valid tick array, return the min/max tick index
                    if a_to_b && next_array.is_min_tick_array() {
                        return Ok((array_index, MIN_TICK_INDEX));
                    } else if !a_to_b && next_array.is_max_tick_array(tick_spacing) {
                        return Ok((array_index, MAX_TICK_INDEX));
                    }

                    // If we are at the last tick array in the sequencer, return the last tick
                    if array_index + 1 == self.arrays.len() {
                        if a_to_b {
                            return Ok((array_index, next_array.start_tick_index()));
                        } else {
                            let last_tick = next_array.start_tick_index() + ticks_in_array - 1;
                            return Ok((array_index, last_tick));
                        }
                    }

                    // No initialized index found. Move the search-index to the 1st search position
                    // of the next array in sequence.
                    search_index = if a_to_b {
                        next_array.start_tick_index() - 1
                    } else {
                        next_array.start_tick_index() + ticks_in_array - 1
                    };

                    array_index += 1;
                }
            }
        }
    }
}
