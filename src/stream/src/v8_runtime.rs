use common::types::TypedValue;
use std::collections::BTreeMap;
use v8::Local;

pub struct RuntimeEngine<'s, 'i> {
    context_scope: v8::ContextScope<'i, v8::HandleScope<'s>>,
    ctx: Local<'s, v8::Context>,
    process_fn: Option<Local<'s, v8::Function>>,
}

impl<'s, 'i> RuntimeEngine<'s, 'i>
where
    's: 'i,
{
    pub fn new(
        source_code: &str,
        fn_name: &str,
        isolated_scope: &'i mut v8::HandleScope<'s, ()>,
    ) -> Self {
        let ctx = v8::Context::new(isolated_scope);
        let mut scope = v8::ContextScope::new(isolated_scope, ctx);
        let code = v8::String::new(&mut scope, source_code).unwrap();

        let script = v8::Script::compile(&mut scope, code, None).unwrap();

        let mut self_ = Self {
            context_scope: scope,
            ctx,
            process_fn: None,
        };

        self_.execute_script(script);

        let process_str = v8::String::new(&mut self_.context_scope, fn_name);

        let fn_value = ctx
            .global(&mut self_.context_scope)
            .get(&mut self_.context_scope, process_str.unwrap().into())
            .unwrap();
        let fn_opt = v8::Local::<v8::Function>::try_from(fn_value);
        let process_fn = if fn_opt.is_ok() {
            Some(fn_opt.unwrap())
        } else {
            None
        };

        self_.process_fn = process_fn;
        self_
    }

    pub(crate) fn call_fn(&mut self, typed_val: TypedValue) -> Option<TypedValue> {
        let scope = &mut v8::HandleScope::new(&mut self.context_scope);
        let ref mut try_catch = v8::TryCatch::new(scope);
        let global = self.ctx.global(try_catch).into();
        let process_fn = self.process_fn.as_mut().unwrap();

        match process_fn.call(try_catch, global, &[wrap_value(typed_val)]) {
            Some(v) => to_typed_value(v),
            None => None,
        }
    }

    fn execute_script(&mut self, script: Local<'s, v8::Script>) {
        let handle_scope = &mut v8::HandleScope::new(&mut self.context_scope);
        let try_catch = &mut v8::TryCatch::new(handle_scope);

        if script.run(try_catch).is_none() {
            let exception = try_catch.exception().unwrap();
            let exception_string = exception
                .to_string(try_catch)
                .unwrap()
                .to_rust_string_lossy(try_catch);

            panic!("{}", exception_string);
        }
    }
}

fn wrap_value(typed_val: TypedValue) -> v8::Local<'static, v8::Value> {
    todo!()
}

fn to_typed_value(local: v8::Local<v8::Value>) -> Option<TypedValue> {
    let ref mut isolate = v8::Isolate::new(Default::default());
    let ref mut scope = v8::HandleScope::new(isolate);
    let ctx = v8::Context::new(scope);
    let context_scope = &mut v8::ContextScope::new(scope, ctx);
    let handle_scope = &mut v8::HandleScope::new(context_scope);
    if local.is_big_int() {
        return local
            .to_big_int(handle_scope)
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
            .number_value(handle_scope)
            .map(|val| TypedValue::Number(val));
    }

    if local.is_null() {
        return Some(TypedValue::Null);
    }
    if local.is_boolean() {
        return Some(TypedValue::Boolean(local.boolean_value(handle_scope)));
    }
    if local.is_string() {
        return local
            .to_string(handle_scope)
            .map(|val| TypedValue::String(val.to_rust_string_lossy(handle_scope)));
    }
    if local.is_object() {
        let args = v8::GetPropertyNamesArgsBuilder::default()
            .key_conversion(v8::KeyConversionMode::ConvertToString)
            .build();
        return local.to_object(handle_scope).and_then(|obj| {
            obj.get_property_names(handle_scope, args).map(|names| {
                let mut map = BTreeMap::default();
                let arr = &*names;
                for index in 0..arr.length() {
                    arr.get_index(handle_scope, index).iter().for_each(|key| {
                        let value = obj.get(handle_scope, key.clone()).unwrap();
                        let v = to_typed_value(value.clone()).unwrap();
                        map.insert(key.to_rust_string_lossy(handle_scope), v.get_data());
                    })
                }
                TypedValue::Object(map)
            })
        });
    }
    Some(TypedValue::Invalid)
}
