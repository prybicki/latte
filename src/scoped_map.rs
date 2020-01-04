use std::collections::HashMap;

pub struct ScopedMap<K,V> {
    scopes: Vec<HashMap<K,V>>,
}

impl<K,V> ScopedMap<K,V>
    where K: std::cmp::Eq + std::hash::Hash
{
    pub fn new() -> ScopedMap<K,V> {
        ScopedMap {scopes: Vec::new()}
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::<K, V>::new())
    }

    pub fn pop_scope(&mut self) -> Option<HashMap<K,V>> {
        self.scopes.pop()
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

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        if self.scopes.last().is_none() {
            self.push_scope();
        }
        let top_scope = self.scopes.last_mut().unwrap();
        return top_scope.insert(key, val);
    }
}