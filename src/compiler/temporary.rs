
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use crate::bytecode::operand::{self, AsDestination, AsSource};
use crate::bytecode::instruction::Instruction;

#[derive(Debug)]
pub struct Temporary {
  manager: Rc<RefCell<InnerTempManager>>,
  id: usize,
}

impl Clone for Temporary {
  fn clone(&self) -> Self {
    let manager = Rc::clone(&self.manager);
    let id = self.id;
    if let Some(use_count) = manager.borrow_mut().temps_use_count.get_mut(id) {
      *use_count += 1;
    }
    Temporary { manager, id }
  }
}

impl Drop for Temporary {
  fn drop(&mut self) {
    if let Some(use_count) = self.manager.borrow_mut().temps_use_count.get_mut(self.id) {
      *use_count -= 1;
    }
  }
}

impl fmt::Display for Temporary {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "R?")
  }
}

impl AsSource for Temporary {
  fn as_first(self) -> Instruction {
    let mut manager = self.manager.borrow_mut();
    let instruction_offset = manager.next_instruction_offset;
    let bit_offset = operand::FIRST_SOURCE_OFFSET;
    manager.deferred_temps.push(DeferredTemp {
      instruction_offset,
      bit_offset,
      id: self.id,
    });
    Instruction::empty()
  }
  fn as_second(self) -> Instruction {
    let mut manager = self.manager.borrow_mut();
    let instruction_offset = manager.next_instruction_offset;
    let bit_offset = operand::SECOND_SOURCE_OFFSET;
    manager.deferred_temps.push(DeferredTemp {
      instruction_offset,
      bit_offset,
      id: self.id,
    });
    Instruction::empty()
  }
}

impl AsDestination for Temporary {
  fn as_destination(self) -> Instruction {
    let mut manager = self.manager.borrow_mut();
    let instruction_offset = manager.next_instruction_offset;
    let bit_offset = operand::DESTINATION_OFFSET;
    manager.deferred_temps.push(DeferredTemp {
      instruction_offset,
      bit_offset,
      id: self.id,
    });
    Instruction::empty()
  }
}

#[derive(Debug)]
struct DeferredTemp {
  pub instruction_offset: usize,
  pub bit_offset: u32,
  pub id: usize,
}

#[derive(Debug)]
struct InnerTempManager {
  temps_use_count: Vec<usize>,
  deferred_temps: Vec<DeferredTemp>,
  next_instruction_offset: usize,
}

#[derive(Debug)]
pub struct TempManager {
  inner: Rc<RefCell<InnerTempManager>>,
}

impl TempManager {
  pub fn new(starting_instruction_offset: usize) -> Self {
    Self {
      inner: Rc::new(RefCell::new(InnerTempManager {
        temps_use_count: Vec::new(),
        deferred_temps: Vec::new(),
        next_instruction_offset: starting_instruction_offset,
      }))
    }
  }
  pub fn count(&self) -> usize {
    self.inner.borrow().temps_use_count.len()
  }
  pub fn reconcile_deferred_temps(&self, temp_registers_start: u8, bytecode: &mut Vec<Instruction>) {
    for deferred in self.inner.borrow().deferred_temps.iter() {
      let temp_register = (temp_registers_start + deferred.id as u8) as u32;
      bytecode[deferred.instruction_offset].0 |= temp_register << deferred.bit_offset;
    }
  }
  pub fn take_temp(&mut self) -> Temporary {
    let manager = Rc::clone(&self.inner);
    let mut borrowed = self.inner.borrow_mut();
    let temps_use_count = &mut borrowed.temps_use_count;
    for (id, use_count) in temps_use_count.iter_mut().enumerate() {
      if *use_count == 0 {
        *use_count += 1;
        return Temporary { manager, id };
      }
    }
    let next_id = temps_use_count.len();
    temps_use_count.push(1);
    Temporary { manager, id: next_id }
  }
  pub fn take_temp_range(&mut self, total: usize) -> Vec<Temporary> {
    let mut inner = self.inner.borrow_mut();
    let temps_use_count = &mut inner.temps_use_count;
    let mut start = 0;
    let mut end = 0;
    while start < temps_use_count.len() {
      if temps_use_count[start] > 0 { // iterate until a free temp is found
        start += 1;
        continue;
      }
      end = start + 1;
      while end < temps_use_count.len() { // iterate until `total` free temps are found
        if end - start + 1 == total { // end is inclusive, so add 1 for length
          let mut temps = Vec::new();
          for id in start..=end {
            temps.push(Temporary {
              manager: Rc::clone(&self.inner),
              id,
            });
            temps_use_count[id] += 1;
          }
          return temps;
        }
        if temps_use_count[end] > 0 { // in use temp found, try again
          start = end + 1;
          continue;
        }
        end += 1;
      }
      start += 1;
    }
    let subtotal = temps_use_count.len() - start;
    let remaining_needed = total - subtotal;
    temps_use_count.resize(temps_use_count.len() + remaining_needed, 1);
    let mut temps = Vec::new();
    for id in start..temps_use_count.len() {
      temps.push(Temporary {
        manager: Rc::clone(&self.inner),
        id,
      });
    }
    temps
  }
  pub fn set_next_instruction_offset(&mut self, next_instruction_offset: usize) {
    self.inner.borrow_mut().next_instruction_offset = next_instruction_offset;
  }
}

