use common::types::TypedValue;
use std::collections::BTreeMap;
use v8::{HandleScope, Local};

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

    pub fn call_fn(&mut self, typed_val: TypedValue) -> Option<TypedValue> {
        let scope = &mut v8::HandleScope::new(&mut self.context_scope);
        let ref mut try_catch = v8::TryCatch::new(scope);
        let global = self.ctx.global(try_catch).into();
        let process_fn = self.process_fn.as_mut().unwrap();

        let isolate = &mut v8::Isolate::new(Default::default());
        let ref mut handle_scope = v8::HandleScope::new(isolate);

        match process_fn.call(try_catch, global, &[wrap_value(&typed_val, handle_scope)]) {
            Some(v) => to_typed_value(v),
            None => {
                try_catch_log(try_catch);
                None
            }
        }
    }

    fn execute_script(&mut self, script: Local<'s, v8::Script>) {
        let handle_scope = &mut v8::HandleScope::new(&mut self.context_scope);
        let try_catch = &mut v8::TryCatch::new(handle_scope);

        if script.run(try_catch).is_none() {
            try_catch_log(try_catch);
        }
    }
}

fn wrap_value<'s>(
    typed_val: &'s TypedValue,
    scope: &'s mut HandleScope<()>,
) -> v8::Local<'s, v8::Value> {
    match typed_val {
        TypedValue::String(value) => {
            let v8_str = v8::String::new(scope, value.as_str()).unwrap();
            v8::Local::<v8::Value>::from(v8_str)
        }
        TypedValue::BigInt(value) => {
            let ctx = v8::Context::new(scope);
            let context_scope = &mut v8::ContextScope::new(scope, ctx);

            let v8_i64 = v8::BigInt::new_from_i64(context_scope, *value);
            v8::Local::<v8::Value>::from(v8_i64)
        }
        TypedValue::Boolean(value) => {
            let v8_bool = v8::Boolean::new(scope, *value);
            v8::Local::<v8::Value>::from(v8_bool)
        }
        TypedValue::Number(value) => {
            let v8_number = v8::Number::new(scope, *value);
            v8::Local::<v8::Value>::from(v8_number)
        }
        TypedValue::Null => {
            let v8_null = v8::null(scope);
            v8::Local::<v8::Value>::from(v8_null)
        }
        TypedValue::Object(value) => {
            let ctx = v8::Context::new(scope);
            let context_scope = &mut v8::ContextScope::new(scope, ctx);

            let v8_obj = v8::Object::new(context_scope);

            let ref mut isolate = v8::Isolate::new(Default::default());
            let ref mut isolated_scope = v8::HandleScope::new(isolate);
            value.iter().for_each(|(key, value)| {
                let key = v8::String::new(isolated_scope, key.as_str()).unwrap();

                let value = TypedValue::from_vec(value);

                let value = wrap_value(&value, isolated_scope);
                v8_obj.create_data_property(
                    context_scope,
                    v8::Local::<v8::Name>::try_from(key).unwrap(),
                    value,
                );
            });
            v8::Local::<v8::Value>::from(v8_obj)
        }
        TypedValue::Invalid => {
            let v8_undefined = v8::undefined(scope);
            v8::Local::<v8::Value>::from(v8_undefined)
        }
    }
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
            obj.get_own_property_names(handle_scope, args).map(|names| {
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

fn try_catch_log(try_catch: &mut v8::TryCatch<v8::HandleScope>) {
    let exception = try_catch.exception().unwrap();
    let exception_string = exception
        .to_string(try_catch)
        .unwrap()
        .to_rust_string_lossy(try_catch);
    log::error!("{}", exception_string);
}

mod tests {

    struct SetupGuard {}

    impl Drop for SetupGuard {
        fn drop(&mut self) {}
    }

    fn setup() -> SetupGuard {
        static START: std::sync::Once = std::sync::Once::new();
        START.call_once(|| {
            v8::V8::set_flags_from_string(
                "--no_freeze_flags_after_init --expose_gc --harmony-import-assertions --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls",
              );
                  v8::V8::initialize_platform(v8::new_default_platform(0, false).make_shared());
                  v8::V8::initialize();
        });

        SetupGuard {}
    }

    #[test]
    fn test_to_typed_value() {
        use common::types::TypedValue;
        use proto::common::common::DataTypeEnum;
        let _setup_guard = setup();

        let isolate = &mut v8::Isolate::new(Default::default());
        let ref mut scope = v8::HandleScope::new(isolate);

        let ctx = v8::Context::new(scope);
        let context_scope = &mut v8::ContextScope::new(scope, ctx);
        let ref mut scope1 = v8::HandleScope::new(context_scope);

        let isolate = &mut v8::Isolate::new(Default::default());
        let ref mut scope = v8::HandleScope::new(isolate);
        let scope2 = &mut v8::HandleScope::new(scope);

        let l1 = v8::BigInt::new_from_i64(scope1, 123);
        let l2 = v8::Number::new(scope2, 78.9);
        let l3 = v8::Local::<v8::BigInt>::try_from(l1).unwrap();
        let l4 = v8::String::new(scope2, "test").unwrap();
        let l5 = v8::null(scope2);
        let l6 = v8::undefined(scope2);
        let l7 = v8::Object::new(scope1);
        // let l8 = v8::Object::new(scope1);

        // let key = v8::String::new(scope2, "key").unwrap();
        // let value = v8::String::new(scope2, "value").unwrap();
        // l8.create_data_property(
        //     scope1,
        //     v8::Local::<v8::Name>::try_from(key).unwrap(),
        //     v8::Local::<v8::Value>::try_from(value).unwrap(),
        // );

        let number_l1 = v8::Local::<v8::Value>::try_from(l2).unwrap();
        let bigint_l1 = v8::Local::<v8::Value>::try_from(l3).unwrap();
        let string_l1 = v8::Local::<v8::Value>::try_from(l4).unwrap();
        let null_l1 = v8::Local::<v8::Value>::try_from(l5).unwrap();
        let undefined_l1 = v8::Local::<v8::Value>::try_from(l6).unwrap();
        let object_l1 = v8::Local::<v8::Value>::try_from(l7).unwrap();
        // let object_l2 = v8::Local::<v8::Value>::try_from(l8).unwrap();

        {
            let value = super::to_typed_value(number_l1);
            assert!(value.is_some());
            let unwrapped_val = value.unwrap();
            assert_eq!(
                unwrapped_val.get_type(),
                DataTypeEnum::DATA_TYPE_ENUM_NUMBER
            );
            match unwrapped_val {
                TypedValue::Number(v) => assert_eq!(v, 78.9),
                _ => panic!("unexpected type"),
            };
        }

        {
            let value = super::to_typed_value(bigint_l1);
            assert!(value.is_some());
            let unwrapped_val = value.unwrap();
            assert_eq!(
                unwrapped_val.get_type(),
                DataTypeEnum::DATA_TYPE_ENUM_BIGINT
            );
            match unwrapped_val {
                TypedValue::BigInt(v) => assert_eq!(v, 123),
                _ => panic!("unexpected type"),
            };
        }

        {
            let value = super::to_typed_value(string_l1);
            assert!(value.is_some());
            let unwrapped_val = value.unwrap();
            assert_eq!(
                unwrapped_val.get_type(),
                DataTypeEnum::DATA_TYPE_ENUM_STRING
            );
            match unwrapped_val {
                TypedValue::String(v) => assert_eq!(v, "test"),
                _ => panic!("unexpected type"),
            };
        }

        {
            let value = super::to_typed_value(null_l1);
            assert!(value.is_some());
            let unwrapped_val = value.unwrap();
            assert_eq!(unwrapped_val.get_type(), DataTypeEnum::DATA_TYPE_ENUM_NULL);
        }

        {
            let value = super::to_typed_value(undefined_l1);
            assert!(value.is_some());
            let unwrapped_val = value.unwrap();
            assert_eq!(
                unwrapped_val.get_type(),
                DataTypeEnum::DATA_TYPE_ENUM_UNSPECIFIED
            );
        }

        {
            let value = super::to_typed_value(object_l1);
            assert!(value.is_some());
            let unwrapped_val = value.unwrap();
            assert_eq!(
                unwrapped_val.get_type(),
                DataTypeEnum::DATA_TYPE_ENUM_OBJECT
            );
            match unwrapped_val {
                TypedValue::Object(v) => assert!(v.is_empty()),
                _ => panic!("unexpected type"),
            }
        }

        {
            // let value = super::to_typed_value(object_l2);
            // assert!(value.is_some());
            // let unwrapped_val = value.unwrap();
            // assert_eq!(
            //     unwrapped_val.get_type(),
            //     DataTypeEnum::DATA_TYPE_ENUM_OBJECT
            // );
            // match unwrapped_val {
            //     TypedValue::Object(v) => {
            //         assert!(!v.is_empty());
            //         assert_eq!(
            //             v.get(&"key".to_string()).map(|data| data.clone()),
            //             Some(TypedValue::String("value".to_string()).get_data())
            //         )
            //     }
            //     _ => panic!("unexpected type"),
            // }
        }
    }

    #[test]
    fn test_v8_runtime_new() {
        use super::RuntimeEngine;
        let _setup_guard = setup();
        let ref mut isolate = v8::Isolate::new(Default::default());
        let ref mut isolated_scope = v8::HandleScope::new(isolate);
        let _rt_engine = RuntimeEngine::new(
            "function process(a, b) { return a+b }",
            "process",
            isolated_scope,
        );
        assert!(_rt_engine.process_fn.is_some())
    }
}
