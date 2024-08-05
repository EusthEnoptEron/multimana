use std::marker::PhantomData;

include!(concat!(env!("OUT_DIR"), "/generated_code.rs"));

pub struct TArray<T> {
    phantom: PhantomData<T>,

}

pub struct TSparseArray<T> {
    phantom: PhantomData<T>,
}

pub struct TSet<T> {
    phantom: PhantomData<T>,
}


pub struct TMap<T1, T2> {
    phantom: PhantomData<T1>,
    phantom2: PhantomData<T2>,
}
pub struct TPair<T1, T2> {
    phantom: PhantomData<T1>,
    phantom2: PhantomData<T2>,
}


pub struct FUObjectItem {}

pub struct TUObjectArray {}

pub struct FNumberedData {}

pub struct FNameEntryHeader {}

pub struct FNameEntry {}

pub struct FName {}
pub struct FString {}

pub struct TSubclassOf<UClass> {
    phantom: PhantomData<UClass>,

}

pub struct FText {}

pub struct FWeakObjectPtr {}

pub struct TWeakObjectPtr<UEType> {
    phantom: PhantomData<UEType>,
}

pub struct TLazyObjectPtr<T> {
    phantom: PhantomData<T>,

}


pub struct TScriptInterface<T> {
    phantom: PhantomData<T>,
}

pub struct FMulticastSparseDelegateProperty_ {}

pub struct FMulticastInlineDelegateProperty_ {}

pub struct FDelegateProperty_ {}

pub struct TSoftObjectPtr<T> {
    phantom: PhantomData<T>,

}

pub struct TSoftClassPtr<T> {
    phantom: PhantomData<T>,
}

pub struct TSoftClassPath<T> {
    phantom: PhantomData<T>,
}

pub struct TFieldPath<T> {
    phantom: PhantomData<T>,
}

pub struct FProperty {}