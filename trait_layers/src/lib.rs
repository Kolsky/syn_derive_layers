#![no_std]

pub unsafe trait Layers: Sized {
    const COUNT: usize;
    fn as_num(&self) -> usize;
    fn try_from_num(num: usize) -> Option<Self>;
}

pub trait Root: Layers {}

pub struct LayersIter<L: Layers> {
    layer: Option<L>,
}

impl<L: Layers> Iterator for LayersIter<L> {
    type Item = L;

    fn next(&mut self) -> Option<Self::Item> {
        let num = self.layer.as_ref()?.as_num().checked_add(1)?;
        core::mem::replace(&mut self.layer, L::try_from_num(num))
    }
}