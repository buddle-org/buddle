use buddle_object_property::cpp::Ptr;
use buddle_object_property::Type;

#[test]
fn test_cast() {
    #[derive(Debug, Default, Type)]
    struct Another {
        #[property()]
        test: Ptr<Test>,
    }

    #[derive(Debug, Default, Type)]
    struct Test {
        #[property()]
        another: u32,
    }

    let test = Test { another: 5 };
    let test_ptr = Ptr::try_new(Box::new(test)).unwrap();
    let another = Another { test: test_ptr };

    assert_eq!(another.test.get().unwrap().another, 5);
}

#[test]
fn test_vec() {
    #[derive(Debug, Default, Type)]
    struct Another {
        #[property()]
        data: Vec<u32>,
    }

    #[derive(Debug, Default, Type)]
    struct Test {
        #[property()]
        test: Vec<Ptr<Another>>,
    }

    let another = Another {
        data: vec![1, 2, 3],
    };
    let test = Test {
        test: vec![Ptr::try_new(Box::new(another)).unwrap()],
    };

    assert_eq!(
        test.test.first().unwrap().get().unwrap().data,
        vec![1, 2, 3]
    )
}
