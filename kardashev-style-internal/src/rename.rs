use std::{
    collections::HashMap,
    convert::Infallible,
};

use lightningcss::{
    selector::{
        Component,
        Selector,
    },
    visit_types,
    visitor::{
        Visit,
        VisitTypes,
        Visitor,
    },
};

#[derive(Debug)]
pub struct RenameClassNames<'a> {
    pub class_names: &'a mut HashMap<String, String>,
    pub file_id: &'a str,
    pub crate_name: &'a str,
}

impl<'a, 'i> Visitor<'i> for RenameClassNames<'a> {
    type Error = Infallible;

    fn visit_types(&self) -> VisitTypes {
        visit_types!(SELECTORS)
    }

    fn visit_selector(&mut self, selectors: &mut Selector<'i>) -> Result<(), Self::Error> {
        for selector in selectors.iter_mut_raw_match_order() {
            match selector {
                Component::Class(ident) => {
                    let new_class_name = format!("{}-{}-{}", self.crate_name, ident, self.file_id);
                    self.class_names
                        .insert(ident.to_string(), new_class_name.clone());
                    *ident = new_class_name.into();
                }
                Component::Slotted(selector) => selector.visit(self)?,
                Component::Host(Some(selector)) => selector.visit(self)?,
                Component::Negation(selector)
                | Component::Where(selector)
                | Component::Is(selector)
                | Component::Any(_, selector)
                | Component::Has(selector) => {
                    selector
                        .iter_mut()
                        .try_for_each(|selector| selector.visit(self))?
                }
                _ => (),
            }
        }

        Ok(())
    }
}
