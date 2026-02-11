use std::marker::PhantomData;
use core_types::indexing_mode::IndexingMode;
use core_types::indexing_mode::IndexingMode::InsertionHeavy;
use core_types::indexing_mode::IndexingMode::*;
use crate::config::Config;

#[derive(Debug)]
pub struct ConfigInner<T>{
    flavor: IndexingMode,
    _marker: PhantomData<T>,
}

impl<T> Default for ConfigInner<T>{
    fn default() -> Self{
        return Self{
            flavor: InsertionHeavy,
            _marker: PhantomData,
        }
    }
}

impl<T> ConfigInner<T>{
    pub(crate) fn new(flavor: IndexingMode) -> Self{
        Self {
            flavor,
            _marker: PhantomData,
        }
    }
}