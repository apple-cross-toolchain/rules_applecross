import SwiftCompilerPlugin
import SwiftSyntaxMacros

@main
struct SwiftUIMacrosPlugin: CompilerPlugin {
    let providingMacros: [Macro.Type] = [
        StateMacro.self,
        StatePropertyWrapperStorageMacro.self,
        StateProjectedValueMacro.self,
        StateInitialStoredValueMacro.self,
        ProjectedValueMacro.self,
    ]
}
