// NATIVE RUST RECONSTRUCTION OF V8 TORQUE LOGIC
// Inherited from: src/builtins/array-map.tq

#[allow(dead_code)]
pub struct JsArray {
    // In No-Chromium, we represent the internal elements using Rust's Option
    // to handle 'Holes' with zero overhead.
    pub elements: Vec<Option<usize>>, 
}

impl JsArray {
    pub fn new(capacity: usize) -> Self {
        Self {
            elements: Vec::with_capacity(capacity),
        }
    }

    /// Native implementation of ArrayMapLoopContinuation (V8 Torque lines 62-94)
    /// This is the 'Sovereign' Rust version: safer and faster.
    pub fn map_native<F>(&self, callback: F) -> Self 
    where 
        F: Fn(usize, usize) -> usize, // callback(value, index) -> new_value
    {
        let mut result_elements = Vec::with_capacity(self.elements.len());
        
        // V8 handles holes by checking 'kPresent' (line 74 in .tq)
        // Rust handles this elegantly with pattern matching on Option.
        for (index, element) in self.elements.iter().enumerate() {
            match element {
                Some(value) => {
                    // Step 81-83 of V8 ADN: Call(callbackfn, T, kValue, k, O)
                    let mapped_value = callback(*value, index);
                    
                    // Step 86 of V8 ADN: FastCreateDataProperty(array, k, mappedValue)
                    result_elements.push(Some(mapped_value));
                }
                None => {
                    // V8 skips the callback if element is a hole (line 77 in .tq)
                    result_elements.push(None);
                }
            }
        }

        Self { elements: result_elements }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v8_inherited_map() {
        let arr = JsArray {
            elements: vec![Some(10), None, Some(30)],
        };
        
        // Simulate a JS callback: x => x * 2
        let result = arr.map_native(|val, _idx| val * 2);
        
        assert_eq!(result.elements[0], Some(20));
        assert_eq!(result.elements[1], None); // Hole preserved
        assert_eq!(result.elements[2], Some(60));
    }
}
