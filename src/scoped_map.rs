use std::collections::HashMap;
use std::collections::hash_map::Keys;

pub type Scope<K,V> = HashMap<K,V>;

#[derive(Clone,Debug)]
pub struct ScopedMap<K,V>
    where K: std::cmp::Eq + std::hash::Hash + std::clone::Clone
{
    scopes: Vec<Scope<K,V>>,
    counter: HashMap<K, isize>,
}

impl<K,V> ScopedMap<K,V>
    where K: std::cmp::Eq + std::hash::Hash + std::clone::Clone
{

    pub fn new() -> ScopedMap<K,V> {
        ScopedMap {scopes: Vec::new(), counter: HashMap::new()}
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::<K, V>::new())
    }

    pub fn pop_scope(&mut self) -> Option<Scope<K,V>> {
        let scope = self.scopes.pop();
        if let Some(scope) = &scope {
            for key in scope.keys() {
                self.change_counter(key.clone(), -1);
            }
        }
        return scope;
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        for scope in self.scopes.iter().rev() {
            let x = scope.get(key);
            if x.is_some() {
                return x
            }
        }
        return None
    }

    fn change_counter(&mut self, key: K, diff: isize) -> () {
        let counter = self.counter.get_mut(&key);
        match counter {
            Some(val) => {
                *val += diff;
                if *val == 0 {
                    self.counter.remove(&key);
                }
            },
            None if diff > 0 => {self.counter.insert(key, diff);},
            _ => panic!("cannot decrease refcount of non-exisiting key"),
        };

    }

    pub fn insert_into_top_scope(&mut self, key: K, val: V) -> Option<V> {
        if self.scopes.last().is_none() {
            self.push_scope();
        }
        let top_scope = self.scopes.last_mut().unwrap();
        let prev = top_scope.insert(key.clone(), val);

        //  first time in this scope, increment occurrences
        if let None = prev {
            self.change_counter(key, 1);
        }
        return prev;
    }

    pub fn replace_topmost(&mut self, key: K, val: V) -> V {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&key) {
                return scope.insert(key, val).unwrap();
            }
        }
        panic!("cannot replace value of non-existing key")
    }

    pub fn keys(&self) -> Keys<K, isize > {
        self.counter.keys()
    }

    pub fn len(&self) -> usize {
        self.scopes.iter().fold(0, |acc, x| acc + x.len())
    }
}