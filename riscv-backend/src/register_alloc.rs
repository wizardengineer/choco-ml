use crate::machine_ir::{MachineFunc, VReg};
use std::{cmp, collections::HashMap};

/// So far we're going to use Linear Scan for doing register allocation.
/// TODO: Implementing Graph coloring...somewhere in the near future

#[derive(Debug, Default, Clone)]
pub struct Interval {
    pub start: usize,
    pub end: usize,
    pub phy_reg: Option<VReg>,
    pub mark_spilled: bool,
}

#[derive(Debug, Clone)]
pub struct LiveIntervals {
    pub vreg: VReg,
    pub start: usize,
    pub end: usize,
    pub phy_reg: Option<VReg>,
    pub mark_spilled: bool,
}

const ALL_REGS: &[VReg] = &[
    // Temp registers
    VReg::T0,
    VReg::T1,
    VReg::T2,
    VReg::T3,
    VReg::T4,
    VReg::T5,
    VReg::T6,
    // Function arguments
    VReg::A0, // function argument 0 / return value 0
    VReg::A1, // function argument 1 / return value 1
    VReg::A2,
    VReg::A3,
    VReg::A4,
    VReg::A5,
    VReg::A6,
    VReg::A7,
    // Saved registers
    //VReg::S0, // frame pointer
    VReg::S1,
    VReg::S2,
    VReg::S3,
    VReg::S4,
    VReg::S5,
    VReg::S6,
    VReg::S7,
    VReg::S8,
    VReg::S9,
    VReg::S10,
    VReg::S11,
    // Return address, Stack pointer & Frame pointer
    //VReg::RA,
    //VReg::SP,
    //VReg::FP,
    // Global Register
    //VReg::GP,
];

#[derive(Debug, Default)]
pub struct LinearScan {
    pub live_intervals: HashMap<VReg, LiveIntervals>,
}

impl LinearScan {
    pub fn new() -> Self {
        Self {
            live_intervals: HashMap::new(),
        }
    }

    pub fn run(&mut self, funcs: &[MachineFunc]) -> HashMap<String, HashMap<VReg, LiveIntervals>> {
        let mut func_by_intervals = HashMap::new();
        for func in funcs.iter() {
            let mut interval = self.build_intervals(func);
            func_by_intervals.insert(func.name.clone(), self.linear_scan(&mut interval));
        }

        func_by_intervals
    }

    pub fn build_intervals(&mut self, mf: &MachineFunc) -> HashMap<VReg, Interval> {
        let mut intervals: HashMap<VReg, Interval> = HashMap::new();

        let mut instrs_global_pos = HashMap::new();
        let mut instr_pos = 0;
        for (b_idx, block) in mf.blocks.iter().enumerate() {
            for i in 0..block.instrs.len() {
                instrs_global_pos.insert((b_idx, i), instr_pos);
                instr_pos += 1;
            }
        }

        for (b_idx, block) in mf.blocks.iter().enumerate() {
            for (i, instr) in block.instrs.iter().enumerate() {
                let pos = instrs_global_pos.get(&(b_idx, i)).unwrap();

                for def in instr.defs() {
                    let interval = intervals.entry(def).or_insert(Interval {
                        start: *pos,
                        end: *pos,
                        mark_spilled: false,
                        phy_reg: None,
                    });

                    interval.start = cmp::min(interval.start, *pos);
                }

                for u in instr.uses() {
                    let interval = intervals.entry(u).or_insert(Interval {
                        start: *pos,
                        end: *pos,
                        mark_spilled: false,
                        phy_reg: None,
                    });

                    interval.end = cmp::max(interval.end, *pos);
                }
            }
        }
        intervals
    }

    pub fn linear_scan(
        &mut self,
        intervals: &mut HashMap<VReg, Interval>,
    ) -> HashMap<VReg, LiveIntervals> {
        // Store our intervals in our Live Intervals sort intervals
        let mut live_intervals: Vec<LiveIntervals> = intervals
            .iter()
            .map(|(vreg, interval)| LiveIntervals {
                vreg: *vreg,
                start: interval.start,
                end: interval.end,
                phy_reg: None,
                mark_spilled: false,
            })
            .collect();

        live_intervals.sort_by_key(|ivl| ivl.start);

        let mut active_alloc_intervals: Vec<LiveIntervals> = Vec::new();
        let mut free_regs = ALL_REGS.to_vec();

        for curr_iv in live_intervals.iter_mut() {
            active_alloc_intervals.retain(|old_iv| {
                if old_iv.end < curr_iv.start {
                    // free old one, to be used later for another Virtual register
                    if let Some(reg) = old_iv.phy_reg {
                        free_regs.push(reg);
                    }
                    false
                } else {
                    true
                }
            });

            // allocate or spill
            if let Some(reg) = free_regs.pop() {
                curr_iv.phy_reg = Some(reg);
                active_alloc_intervals.push(curr_iv.clone());

                // sort by increasing order for end of a interval
                active_alloc_intervals.sort_by_key(|x| x.end);
            }
            // Spill if we can't get any Free Register (free_regs)
            else {
                let mut worst = active_alloc_intervals.pop().unwrap();

                if worst.end > curr_iv.end {
                    curr_iv.phy_reg = worst.phy_reg.take();
                    curr_iv.mark_spilled = false;
                    worst.mark_spilled = true;
                    active_alloc_intervals.push(curr_iv.clone());
                    active_alloc_intervals.sort_by_key(|x| x.end);
                } else {
                    curr_iv.mark_spilled = true;
                    active_alloc_intervals.push(worst);
                }
            }

            if let Some(entry) = intervals.get_mut(&curr_iv.vreg) {
                entry.phy_reg = curr_iv.phy_reg;
                entry.mark_spilled = curr_iv.mark_spilled;
            }
        }

        live_intervals.into_iter().map(|iv| (iv.vreg, iv)).collect()
    }
}
