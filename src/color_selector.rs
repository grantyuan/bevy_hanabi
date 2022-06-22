/// color selector 
/// 
use bevy::utils::HashMap;

/// value range
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueRange {
    /// start
    pub start: f32,
    /// end
    pub end: f32,
}

impl Eq for ValueRange {}

impl std::hash::Hash for ValueRange {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.start.to_string() + &self.end.to_string()).hash(state);
    }
}

/// which index shoud apply the rule
#[derive(Debug, Clone, PartialEq)]
pub enum Indicator {
    /// particles' speed
    SPEED,
    // DIRECTION,
    // / particles' size
    // SIZE,
}

/// color selector
#[derive(Debug, Clone, PartialEq)]
pub struct ColorSelector<T> {
    /// var name in the range
    pub depend_var_name: Indicator,

    /// range values
    pub range_values: HashMap<ValueRange, T>,
}

impl<T> Default for ColorSelector<T> {
    fn default() -> Self {
        Self { depend_var_name: Indicator::SPEED, range_values: Default::default() }
    }
}

impl<T: Default + Sized + Clone> ColorSelector<T> {
    /// new instance  with the depend_var_name
    pub fn new(depend_var_name: Indicator) -> Self {
        ColorSelector {
            depend_var_name,
            range_values: HashMap::new(),
        }
    }

    /// add range and value
    pub fn add_range(&mut self, range: ValueRange, value: T) -> &mut Self {
        // let arc_range_value = self.range_values.clone();
        let entry = self.range_values.entry(range).or_insert(value.clone());
        *entry = value;
        self
    }
}
