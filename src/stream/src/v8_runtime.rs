use common::types::TypedValue;
use std::collections::BTreeMap;
use v8::Local;

pub struct RuntimeEngine<'s> {
    scope: &'s mut v8::ContextScope<'s, v8::HandleScope<'s>>,
    ctx: Local<'s, v8::Context>,
    script: v8::Local<'s, v8::Script>,
}

impl<'s> RuntimeEngine<'s> {
    pub fn new(source_code: &str) -> Self {
        let ref mut isolate = v8::Isolate::new(Default::default());
        let handle_scope = &mut v8::HandleScope::new(isolate);
        let ctx = v8::Context::new(handle_scope);
        let ref mut scope = v8::ContextScope::new(handle_scope, ctx);
        let code = v8::String::new(scope, source_code).unwrap();
        let script = v8::Script::compile(scope, code, None).unwrap();

        Self { scope, ctx, script }
    }

    pub(crate) fn call_fn(&mut self, fn_name: &str, typed_val: TypedValue) -> Option<TypedValue> {
        self.ctx
            .global(self.scope)
            .get(
                self.scope,
                v8::String::new(self.scope, fn_name).unwrap().into(),
            )
            .map(|value| v8::Local::<v8::Function>::try_from(value))
            .and_then(|func| {
                let ref mut try_catch = v8::TryCatch::new(self.scope);
                func.map(|f| {
                    f.call(
                        try_catch,
                        self.ctx.global(try_catch).into(),
                        &[self.wrap_value(typed_val)],
                    )
                    .map(|v| self.to_typed_value(v))
                    .unwrap_or(None)
                })
                .unwrap_or(None)
            })
    }

    fn wrap_value(&self, typed_val: TypedValue) -> v8::Local<v8::Value> {
        todo!()
    }

    fn to_typed_value(&mut self, local: v8::Local<v8::Value>) -> Option<TypedValue> {
        if local.is_big_int() {
            return local
                .to_big_int(&mut self.scope)
                .filter(|val| {
                    let (_, ok) = val.i64_value();
                    ok
                })
                .map(|val| {
                    let (v, _) = val.i64_value();
                    TypedValue::BigInt(v)
                });
        }
        if local.is_number() {
            return local
                .number_value(&mut self.scope)
                .map(|val| TypedValue::Number(val));
        }

        if local.is_null() {
            return Some(TypedValue::Null);
        }
        if local.is_boolean() {
            return Some(TypedValue::Boolean(local.boolean_value(&mut self.scope)));
        }
        if local.is_string() {
            return local
                .to_string(&mut self.scope)
                .map(|val| TypedValue::String(val.to_rust_string_lossy(&mut self.scope)));
        }
        if local.is_object() {
            let args = v8::GetPropertyNamesArgsBuilder::default()
                .key_conversion(v8::KeyConversionMode::ConvertToString)
                .build();
            return local.to_object(&mut self.scope).and_then(|obj| {
                obj.get_property_names(&mut self.scope, args).map(|names| {
                    let map = BTreeMap::default();
                    let arr = *names;
                    for index in 0..arr.length() {
                        arr.get_index(&mut self.scope, index)
                            .iter()
                            .for_each(|key| {
                                let Some(value) = obj.get(&mut self.scope, key.clone());
                                let Some(v) = self.to_typed_value(value.clone());
                                map.insert(key.to_rust_string_lossy(&mut self.scope), v.get_data());
                            })
                    }
                    TypedValue::Object(map)
                })
            });
        }
        Some(TypedValue::Invalid)
    }
}
