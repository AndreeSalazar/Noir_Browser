# THE BIBLE OF V8: RECONSTRUCTION MAP

Total Files Analyzed: 69

## 1. Core Classes (Architecture)
- **BackingStore** (Source: include_v8-array-buffer.h)
- **ArrayBuffer** (Source: include_v8-array-buffer.h)
- **Allocator** (Source: include_v8-array-buffer.h)
- **ArrayBufferView** (Source: include_v8-array-buffer.h)
- **DataView** (Source: include_v8-array-buffer.h)
- **SharedArrayBuffer** (Source: include_v8-array-buffer.h)
- **Array** (Source: include_v8-container.h)
- **Map** (Source: include_v8-container.h)
- **Set** (Source: include_v8-container.h)
- **ExtensionConfiguration** (Source: include_v8-context.h)
- **Context** (Source: include_v8-context.h)
- **V8_NODISCARD** (Source: include_v8-context.h)
- **Data** (Source: include_v8-data.h)
- **FixedArray** (Source: include_v8-data.h)
- **Date** (Source: include_v8-date.h)
- **StackFrame** (Source: include_v8-debug.h)
- **StackTrace** (Source: include_v8-debug.h)
- **EmbedderRootsHandler** (Source: include_v8-embedder-heap.h)
- **EmbedderStateScope** (Source: include_v8-embedder-state-scope.h)
- **Exception** (Source: include_v8-exception.h)

## 2. JS Built-ins (Logic)
### src_builtins_array-filter.tq
- `ArrayFilterLoopContinuation`
- `FastArrayFilter`
- `FastFilterSpeciesCreate`
### src_builtins_array-foreach.tq
- `ArrayForEachLoopContinuation`
- `FastArrayForEach`
### src_builtins_array-map.tq
- `ArrayMapLoopContinuation`
- `ReportSkippedElement`
- `CreateJSArray`
- `StoreResult`
- `NewVector`
### src_builtins_array-reduce.tq
- `ArrayReduceLoopContinuation`
- `FastArrayReduce`
### src_builtins_array-slice.tq
- `HandleSimpleArgumentsSlice`
- `HandleFastAliasedSloppyArgumentsSlice`
- `HandleFastSlice`
### src_builtins_boolean.tq
- `ThisBooleanValue`
### src_builtins_number.tq
- `NaNStringConstant`
- `ZeroStringConstant`
- `InfinityStringConstant`
- `MinusInfinityStringConstant`
- `Log10OffsetTable`
### src_builtins_object.tq
- `ObjectIsExtensibleImpl`
- `ObjectPreventExtensionsThrow`
- `ObjectPreventExtensionsDontThrow`
- `ObjectGetPrototypeOfImpl`
- `JSReceiverGetPrototypeOf`
### src_builtins_promise-abstract-operations.tq
- `PromiseForwardingHandlerSymbolConstant`
- `PromiseHandledBySymbolConstant`
- `ResolveStringConstant`
- `IsPromiseResolveProtectorCellInvalid`
- `AllocateRootFunctionWithContext`
### src_builtins_promise-constructor.tq
- `IsDebugActive`
- `HasAccessCheckFailed`
- `ConstructorBuiltinsAssembler`
- `PromiseBuiltinsAssembler`
