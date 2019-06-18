extern crate pwasm_ethereum;
extern crate pwasm_std;

use pwasm_std::{ Vec, types::{ H256, Address } };

// An entry in the storage table
struct Entry {
    key: H256,
    value: [u8; 32]
}

// Struct for storing an observable value's name
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct ObsName {
    pub name: Vec<i64>
}

impl ObsName {
    // Initialise a new ObsName object
    pub fn new(name: &Vec<i64>) -> ObsName {
        ObsName {
            name: name.to_vec()
        }
    }
}

// Storage table, stores anything looked up while the contract is executing, clears upon exiting contract code
pub struct Storage {
    table: Vec<Entry>
}

// The implementing struct can store values of the given type (passed/returned by value). These values must never change size.
pub trait StoresFixed<T> {
    // Reads a value of the given type from storage, returns the value and the last used address (storage is done sequentially)
    fn read(&mut self, key: &H256) -> (T, H256);

    // Writes a value of the given type to storage, returns the last used address (storage is done sequentially)
    fn write(&mut self, key: &H256, value: T) -> H256;

    fn size() -> usize {
        1
    }
}

// The implementing struct can store a vector of values of the given type (passed/returned by value). These values must never change size.
pub trait StoresFixedVec<T> where Self: StoresFixed<T> + StoresFixed<i64> {
    // Reads a set of values of the given type from storage, returns the value and the last used address (storage is done sequentially)
    fn read_vec(&mut self, key: &H256) -> (Vec<T>, H256);

    // Writes a set of values of the given type to storage, returns the last used address (storage is done sequentially)
    fn write_vec(&mut self, key: &H256, value: &Vec<T>) -> H256;

    // Gets a single element of the given type from storage.
    fn get(&mut self, key: &H256, index: usize) -> T;

    // Sets a single element of the given type in storage.
    fn set(&mut self, key: &H256, index: usize, value: T);

    // Adds an element to the end of the stored vector.
    fn push(&mut self, key: &H256, value: T);

    // Gets the length of the vector
    fn length(&mut self, key: &H256) -> usize;
}

// The implementing struct can store values of the given type (passed by reference). These values can change size.
pub trait StoresVariable<T> {
    // Reads a value of the given type from storage, returns the value and the last used address (storage is done sequentially)
    fn read_var(&mut self, key: &H256) -> (T, H256);

    // Writes a value of the given type to storage (from reference), returns the last used address (storage is done sequentially)
    fn write_var(&mut self, key: &H256, value: &T) -> H256;
}


// Storage method implementation
impl Storage {
    // Initialise a new storage table
    pub fn new() -> Storage {
        Storage { table: Vec::new() }
    }
    
    // Convert a stored value into an address
    fn to_address(value: &[u8; 32]) -> Address {
        H256::from(value).into()
    }

    // Convert an address into a storable value
    fn from_address(value: &Address) -> [u8; 32] {
        H256::from(value.clone()).into()
    }

    // Convert a storable value into an i64
    fn to_i64(value: &[u8; 32]) -> i64 {
        let mut res: i64;

        // Deserialize accounting for endianness
        let unsigned = pwasm_std::read_u64(value);

        // Convert u64 to i64
        if unsigned >= 2_u64.pow(63) {
            // Convert from 2^63..2^64 - 1 to -(2^63 - 1)..0
            res = (unsigned - 2_u64.pow(63)) as i64;
            res -= 2_i64.pow(62);
            res -= 2_i64.pow(62);
        } else {
            // Unsigned int is in same range as signed int (most significant bit is 0)
            res = unsigned as i64
        }

        res
    }

    // Convert an i64 into a storable value
    fn from_i64(mut value: i64) -> [u8; 32] {
        // Convert to u64
        let unsigned;
        if value < 0 {
            // Convert from two's complement negative to unsigned
            value += 2_i64.pow(62);
            value += 2_i64.pow(62);
            unsigned = value as u64 + 2_u64.pow(63);
        } else {
            // Signed int is positive, convert straight to unsigned
            unsigned = value as u64;
        }
        
        // Serialize accounting for endianness
        let mut res: [u8; 32] = [0; 32];
        pwasm_std::write_u64(&mut res, unsigned);
        res
    }

    // Converts a storable value into a bool
    fn to_bool(value: &[u8; 32]) -> bool {
        H256::from(value) != H256::zero()
    }

    // Converts a bool into a storable value
    fn from_bool(value: bool) -> [u8; 32] {
        let mut res: [u8; 32];
        if value {
            res= [0; 32];
            res[0] = 1;
        } else {
            res = H256::zero().into();
        }
        res
    }
}

impl StoresFixed<[u8; 32]> for Storage {
    // Read a value from storage, store locally if not already
    fn read(&mut self, key: &H256) -> ([u8; 32], H256) {
        for entry in &self.table {
            if entry.key == *key {
                return (entry.value.clone(), *key);
            }
        }

        let value = pwasm_ethereum::read(key);
        self.table.push(Entry {
            key: key.clone(),
            value: value.clone()
        });
        (value, *key)
    }

    // Write a value to storage and store locally
    fn write(&mut self, key: &H256, value: [u8; 32]) -> H256 {
        pwasm_ethereum::write(key, &value);

        for entry in &mut self.table {
            if entry.key == *key {
                entry.value = value.clone();
                return *key;
            }
        }

        self.table.push(Entry {
            key: key.clone(),
            value: value.clone()
        });
        *key
    }
}

impl StoresFixed<Address> for Storage {
    fn read(&mut self, key: &H256) -> (Address, H256) {
        let (value, last_used): ([u8; 32], H256) = self.read(key);
        (Storage::to_address(&value), last_used)
    }

    fn write(&mut self, key: &H256, value: Address) -> H256 {
        self.write(key, Storage::from_address(&value));
        *key
    }
}

impl StoresFixed<i64> for Storage {
    fn read(&mut self, key: &H256) -> (i64, H256) {
        let (value, last_used): ([u8; 32], H256) = self.read(key);
        (Storage::to_i64(&value), last_used)
    }

    fn write(&mut self, key: &H256, value: i64) -> H256 {
        self.write(key, Storage::from_i64(value));
        *key
    }
}

impl StoresFixed<u32> for Storage {
    fn read(&mut self, key: &H256) -> (u32, H256) {
        let (value, last_used): ([u8; 32], H256) = self.read(key);
        // Converts from an i64, works as long as the stored value is actually a u32 (should always be the case)
        (Storage::to_i64(&value) as u32, last_used)
    }

    fn write(&mut self, key: &H256, value: u32) -> H256 {
        // Converts to an i64, works as long as the value being stored is 0 < val < 2^63-1, we have 0 < val_u32 < 2^32-1
        self.write(key, Storage::from_i64(value as i64));
        *key
    }
}

impl StoresFixed<bool> for Storage {
    fn read(&mut self, key: &H256) -> (bool, H256) {
        let (value, last_used): ([u8; 32], H256) = self.read(key);
        (Storage::to_bool(&value), last_used)
    }

    fn write(&mut self, key: &H256, value: bool) -> H256 {
        self.write(key, Storage::from_bool(value))
    }
}

impl<T> StoresFixed<Option<T>> for Storage where Storage: StoresFixed<T> + StoresFixed<bool> {
    fn size() -> usize {
        <Storage as StoresFixed<T>>::size() + 1
    }

    fn read(&mut self, key: &H256) -> (Option<T>, H256) {
        let some: bool = <Storage as StoresFixed<bool>>::read(self, key).0;
        if some {
            let (value, last_used) = self.read(&add_to_key(*key, 1));
            (Some(value), last_used)
        } else {
            (None, add_to_key(*key, <Storage as StoresFixed<T>>::size() as u64))
        }
    }

    fn write(&mut self, key: &H256, value: Option<T>) -> H256 {
        match value {
            Some(v) => {
                let last_used = <Storage as StoresFixed<bool>>::write(self, key, true);
                self.write(&add_to_key(last_used, 1), v)
            },
            None => {
                <Storage as StoresFixed<bool>>::write(self, key, false);
                // Cannot have variable-length elements in StoresFixed, so always save 2 slots even if no value to write
                add_to_key(*key, <Storage as StoresFixed<T>>::size() as u64)
            }
        }
    }
}

// Tuple implementation
impl<T, U> StoresFixed<(T, U)> for Storage where Storage: StoresFixed<T> + StoresFixed<U> {
    fn read(&mut self, key: &H256) -> ((T, U), H256) {
        let (first, key0): (T, H256) = self.read(key);
        let (second, key1): (U, H256) = self.read(&add_to_key(key0, 1));
        ((first, second), key1)
    }

    fn write(&mut self, key: &H256, value: (T, U)) -> H256 {
        let key0 = self.write(&key, value.0);
        self.write(&add_to_key(key0, 1), value.1)
    }

    fn size() -> usize {
        <Storage as StoresFixed<T>>::size() + <Storage as StoresFixed<U>>::size()
    }
}

impl<T, U, V> StoresFixed<(T, U, V)> for Storage where Storage: StoresFixed<T> + StoresFixed<U> + StoresFixed<V> {
    fn read(&mut self, key: &H256) -> ((T, U, V), H256) {
        let (first, key0): (T, H256) = self.read(key);
        let (second, key1): (U, H256) = self.read(&add_to_key(key0, 1));
        let (third, key2): (V, H256) = self.read(&add_to_key(key1, 1));
        ((first, second, third), key2)
    }

    fn write(&mut self, key: &H256, value: (T, U, V)) -> H256 {
        let key0 = self.write(&key, value.0);
        let key1 = self.write(&add_to_key(key0, 1), value.1);
        self.write(&add_to_key(key1, 1), value.2)
    }

    fn size() -> usize {
        <Storage as StoresFixed<T>>::size() + <Storage as StoresFixed<U>>::size() + <Storage as StoresFixed<V>>::size()
    }
}


impl<T> StoresFixedVec<T> for Storage where Storage: StoresFixed<T> + StoresFixed<i64>, Vec<T>: core::clone::Clone {
    // Reads a set of values of the given type from storage, returns the value and the last used address (storage is done sequentially)
    fn read_vec(&mut self, key: &H256) -> (Vec<T>, H256) {
        let length: usize = self.length(key);
        let mut current = add_to_key(*key, 1);
        let mut res: Vec<T> = Vec::new();

        let mut last_used: H256 = *key;
        for _ in 0..length {
            let (value, end): (T, H256) = self.read(&current);
            last_used = end;
            res.push(value);
            current = add_to_key(end, 1);
        }

        (res, last_used)
    }

    // Writes a set of values of the given type to storage, returns the last used address (storage is done sequentially)
    fn write_vec(&mut self, key: &H256, value: &Vec<T>) -> H256 {
        let length = value.len();
        <Storage as StoresFixed<i64>>::write(self, key, length as i64);
        let mut last_used = *key;
        let mut clone = value.clone();

        // Consume list clone, writing each element to storage
        for _ in 0..length {
            last_used = self.write(&add_to_key(last_used, 1 as u64), clone.remove(0));
        }
        last_used
    }

    // Gets a single element of the given type from storage.
    fn get(&mut self, key: &H256, index: usize) -> T {
        let length: usize = self.length(key);

        if length <= index {
            panic!("Stored vector index out of bounds.");
        }

        let size = <Storage as StoresFixed<T>>::size();
        let elem_key = &add_to_key(*key, (1 + size * index) as u64);
        self.read(elem_key).0
    }

    // Sets a single element of the given type in storage.
    fn set(&mut self, key: &H256, index: usize, value: T) {
        let length: usize = self.length(key);

        if length <= index {
            panic!("Stored vector index out of bounds.");
        }

        let size = <Storage as StoresFixed<T>>::size();
        let elem_key = &add_to_key(*key, (1 + size * index) as u64);
        self.write(elem_key, value);
    }

    // Adds an element to the end of the stored vector.
    fn push(&mut self, key: &H256, value: T) {
        let length: usize = self.length(key);

        let size = <Storage as StoresFixed<T>>::size();
        let elem_key = &add_to_key(*key, (1 + size * length) as u64);
        self.write(elem_key, value);

        <Storage as StoresFixed<i64>>::write(self, key, (length + 1) as i64);
    }

    // Gets the length of the vector
    fn length(&mut self, key: &H256) -> usize {
        <Storage as StoresFixed<i64>>::read(self, key).0 as usize
    }
}


// Vectors are stored sequentially
impl<T> StoresVariable<Vec<T>> for Storage where Storage: StoresVariable<T> + StoresFixed<i64>, Vec<T>: core::clone::Clone {
    // Reads vector sequentially from storage
    fn read_var(&mut self, key: &H256) -> (Vec<T>, H256) {
        let length: usize = <Storage as StoresFixed<i64>>::read(self, key).0 as usize;
        let mut current = add_to_key(*key, 1);
        let mut res: Vec<T> = Vec::new();        

        let mut last_used: H256 = *key;
        for _ in 0..length {
            let (value, end): (T, H256) = self.read_var(&current);
            last_used = end;
            res.push(value);
            current = add_to_key(end, 1);
        }

        (res, last_used)
    }

    fn write_var(&mut self, key: &H256, value: &Vec<T>) -> H256 {
        let length = value.len();
        <Storage as StoresFixed<i64>>::write(self, key, length as i64);
        let mut last_used = *key;
        let mut clone = value.clone();

        // Consume list clone, writing each element to storage
        for _ in 0..length {
            last_used = self.write_var(&add_to_key(last_used, 1 as u64), &clone.remove(0));
        }
        last_used
    }
}

// ObsName implementation (can store normally as name never changes, so size never changes)
impl StoresVariable<ObsName> for Storage where Storage: StoresFixedVec<i64> {
    fn read_var(&mut self, key: &H256) -> (ObsName, H256) {
        let (name, key) = self.read_vec(key);
        (ObsName::new(&name), key)
    }

    fn write_var(&mut self, key: &H256, value: &ObsName) -> H256 {
        self.write_vec(key, &value.name.clone())
    }
}

// Converts an address to an array of i64s
pub fn address_to_i64(value: Address) -> [i64; 4] {
    let mut res: [i64; 4] = [0; 4];

    // Get address as storable
    let addr: [u8; 32] = Storage::from_address(&value);
    for i in 0..4 {
        // Convert first byte to storable i64
        let mut addr_storable: [u8; 32] = [0; 32];
        for j in 0..8 {
            addr_storable[j] = addr[(i * 8) + j];
        }
        // Convert to i64, and push
        res[i] = Storage::to_i64(&addr_storable);
    }

    res
}

// Converts an array of i64s to an address
pub fn i64_to_address(value: [i64; 4]) -> Address {
    // Get i64s as storable
    let mut vals: [u8; 32] = [0; 32];
    for i in 0..4 {
        // Get i'th i64 as storable
        let mut storable = Storage::from_i64(value[i]);
        for j in 0..8 {
            // Get first 8 bytes of storable i64, store in address
            vals[i * 8 + j] = storable[j];
        }
    }

    Storage::to_address(&vals)
}

// Adds a number to a storage key
fn add_to_key(key: H256, mut value: u64) -> H256 {
    let mut i = 0;
    let mut new_key: [u8; 32] = key.clone().into();
    while value > 0 {
        if value >= 255 - (new_key[i] as u64) {
            value -= 255 - (new_key[i] as u64);
            new_key[i] = 255;
            i += 1;
        } else {
            new_key[i] = new_key[i] + value as u8;
            value = 0;
        }

        // Only allow modification of the first 31 bytes of the address, the last byte separates memory namespaces and cannot be crossed
        if i >= 31 {
            panic!("Storage key overflow.");
        }
    }

    H256::from(new_key)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use pwasm_std::{ vec };

    // Storage size of types which can be stored in a single storage slot are correct.
    #[test]
    fn storage_size_single_correct() {
        assert_eq!(<Storage as StoresFixed<[u8; 32]>>::size(), 1);
        assert_eq!(<Storage as StoresFixed<bool>>::size(), 1);
        assert_eq!(<Storage as StoresFixed<u32>>::size(), 1);
        assert_eq!(<Storage as StoresFixed<i64>>::size(), 1);
        assert_eq!(<Storage as StoresFixed<Address>>::size(), 1);
    }

    // Storage size of Options are correct.
    #[test]
    fn storage_size_option_correct() {
        assert_eq!(<Storage as StoresFixed<Option<i64>>>::size(), 2);
        assert_eq!(<Storage as StoresFixed<Option<Option<i64>>>>::size(), 3);
    }

    // Storage size of Tuples are correct.
    #[test]
    fn storage_size_tuple_correct() {
        assert_eq!(<Storage as StoresFixed<(Option<i64>, u32)>>::size(), 3);
        assert_eq!(<Storage as StoresFixed<(Option<i64>, u32, (bool, Address))>>::size(), 5);
    }

    // Storage of an i64 works correctly
    #[test]
    fn stores_and_retrieves_i64_correctly() {
        let mut storage: Storage = Storage::new();

        let value: i64 = 14436934069;
        storage.write(&H256::zero(), value);
        assert_eq!(storage.read(&H256::zero()), (value, H256::zero()));
    }

    // Storage of a u32 works correctly
    #[test]
    fn stores_and_retrieves_u32_correctly() {
        let mut storage: Storage = Storage::new();

        let value: u32 = 1443693;
        storage.write(&H256::zero(), value);
        assert_eq!(storage.read(&H256::zero()), (value, H256::zero()));
    }

    // Storage of a bool works correctly
    #[test]
    fn stores_and_retrieves_bool_correctly() {
        let mut storage: Storage = Storage::new();

        let value0: bool = true;
        storage.write(&H256::zero(), value0);
        assert_eq!(storage.read(&H256::zero()), (value0, H256::zero()));

        let value1: bool = false;
        storage.write(&H256::zero(), value1);
        assert_eq!(storage.read(&H256::zero()), (value1, H256::zero()));
    }

    // Storage of an Address works correctly
    #[test]
    fn stores_and_retrieves_address_correctly() {
        let mut storage: Storage = Storage::new();

        let value: Address = Address::from([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]);
        storage.write(&H256::zero(), value);
        assert_eq!(storage.read(&H256::zero()), (value, H256::zero()));
    }

    // Storage of an Option works correctly
    #[test]
    fn stores_and_retrieves_option_correctly() {
        let mut storage: Storage = Storage::new();

        let value0: Option<i64> = Some(123);
        storage.write(&H256::zero(), value0);
        assert_eq!(storage.read(&H256::zero()), (value0, add_to_key(H256::zero(), 1)));

        let value1: Option<i64> = None;
        storage.write(&H256::zero(), value1);
        assert_eq!(storage.read(&H256::zero()), (value1, add_to_key(H256::zero(), 1)));
    }

    // Storage of a tuple works correctly
    #[test]
    fn stores_and_retrieves_tuple_correctly() {
        let mut storage: Storage = Storage::new();

        let value0: Option<i64> = Some(123);
        let value1: i64 = 10;
        storage.write(&H256::zero(), (value0, value1));
        assert_eq!(storage.read(&H256::zero()), ((value0, value1), add_to_key(H256::zero(), 2)));
    }

    // Storage of a Vec of StoresFixed-able values works correctly
    #[test]
    fn stores_and_retrieves_fixed_vec_correctly() {
        let mut storage: Storage = Storage::new();

        let value: Vec<i64> = vec![15425, 15436136, 1346134, 123093, 132523];
        storage.write_vec(&H256::zero(), &value.clone());
        assert_eq!(storage.read_vec(&H256::zero()), (value, add_to_key(H256::zero(), 5)));
    }

    // Setting and getting elements in a Vec of StoresFixed-able values works correctly
    #[test]
    fn sets_and_gets_fixed_vec_correctly() {
        let mut storage: Storage = Storage::new();

        let value: Vec<i64> = vec![0, 1, 2, 3, 4];
        storage.write_vec(&H256::zero(), &value.clone());

        let mut gotten: i64 = storage.get(&H256::zero(), 2);
        assert_eq!(gotten, 2);

        storage.set(&H256::zero(), 2, 10 as i64);
        gotten = storage.get(&H256::zero(), 2);
        assert_eq!(gotten, 10);

        let read: Vec<i64> = storage.read_vec(&H256::zero()).0;
        assert_eq!(read, vec![0, 1, 10, 3, 4]);
    }

    // Pushing elements in a Vec of StoresFixed-able values works correctly
    #[test]
    fn pushes_fixed_vec_correctly() {
        let mut storage: Storage = Storage::new();

        let value: Vec<i64> = vec![0, 1];
        storage.write_vec(&H256::zero(), &value.clone());
        
        storage.push(&H256::zero(), 2 as i64);
        storage.push(&H256::zero(), 3 as i64);
        storage.push(&H256::zero(), 4 as i64);
        
        let read: Vec<i64> = storage.read_vec(&H256::zero()).0;
        assert_eq!(read, vec![0, 1, 2, 3, 4]);
    }

    // Storage of an ObsName works correctly
    #[test]
    fn stores_and_retrieves_observable_names_correctly() {
        let mut storage: Storage = Storage::new();

        let value: ObsName = ObsName::new(&vec![5, 4, 3, 2, 1, 0]);
        storage.write_var(&H256::zero(), &value);

        let read: ObsName = storage.read_var(&H256::zero()).0;
        assert_eq!(read, value);
    }

    // Storage of a variable Vec works correctly
    #[test]
    fn stores_and_retrieves_variable_vec_correctly() {
        let mut storage: Storage = Storage::new();

        let value: Vec<ObsName> = vec![ObsName::new(&vec![0, 1, 2, 3])];
        storage.write_var(&H256::zero(), &value);
        assert_eq!(storage.read_var(&H256::zero()), (value, add_to_key(H256::zero(), 5)));
    }

    // Conversion between Address and i64 works correctly
    #[test]
    fn converts_between_i64_and_address_correctly() {
        let arbiter: Address = "3D04E16e08E4c1c7fa8fC5A386237669341EaAcE".parse().unwrap();
        let arbiter_serialized: [i64; 4] = address_to_i64(arbiter);

        assert_eq!(arbiter, i64_to_address(arbiter_serialized));
        assert_eq!(arbiter_serialized, [0, 7989671873971486720, -6645747367859330040, -3554995745399102586]);
    }
}