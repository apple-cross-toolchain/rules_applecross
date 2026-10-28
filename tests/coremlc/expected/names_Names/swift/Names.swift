//
// Names.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class NamesInput : MLFeatureProvider {

    /// x as 1 by 2 matrix of floats
    public var x: MLMultiArray

    public var featureNames: Set<String> { ["x"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "x" {
            return MLFeatureValue(multiArray: x)
        }
        return nil
    }

    public init(x: MLMultiArray) {
        self.x = x
    }

    public convenience init(x: MLShapedArray<Float>) {
        self.init(x: MLMultiArray(x))
    }

}


/// Model Prediction Output Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class NamesOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// out as 1 by 2 matrix of floats
    public var out_: MLMultiArray {
        provider.featureValue(for: "out")!.multiArrayValue!
    }

    /// out as 1 by 2 matrix of floats
    public var out_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(out_)
    }

    /// var as 1 by 2 matrix of floats
    public var var_: MLMultiArray {
        provider.featureValue(for: "var")!.multiArrayValue!
    }

    /// var as 1 by 2 matrix of floats
    public var var_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(var_)
    }

    /// in as 1 by 2 matrix of floats
    public var in_: MLMultiArray {
        provider.featureValue(for: "in")!.multiArrayValue!
    }

    /// in as 1 by 2 matrix of floats
    public var in_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(in_)
    }

    /// inout as 1 by 2 matrix of floats
    public var inout_: MLMultiArray {
        provider.featureValue(for: "inout")!.multiArrayValue!
    }

    /// inout as 1 by 2 matrix of floats
    public var inout_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(inout_)
    }

    /// self as 1 by 2 matrix of floats
    public var self_: MLMultiArray {
        provider.featureValue(for: "self")!.multiArrayValue!
    }

    /// self as 1 by 2 matrix of floats
    public var self_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(self_)
    }

    /// class as 1 by 2 matrix of floats
    public var class_: MLMultiArray {
        provider.featureValue(for: "class")!.multiArrayValue!
    }

    /// class as 1 by 2 matrix of floats
    public var class_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(class_)
    }

    /// func_workaround as 1 by 2 matrix of floats
    public var func_workaround: MLMultiArray {
        provider.featureValue(for: "func_workaround")!.multiArrayValue!
    }

    /// func_workaround as 1 by 2 matrix of floats
    public var func_workaroundShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(func_workaround)
    }

    /// let as 1 by 2 matrix of floats
    public var let_: MLMultiArray {
        provider.featureValue(for: "let")!.multiArrayValue!
    }

    /// let as 1 by 2 matrix of floats
    public var let_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(let_)
    }

    /// import as 1 by 2 matrix of floats
    public var import_: MLMultiArray {
        provider.featureValue(for: "import")!.multiArrayValue!
    }

    /// import as 1 by 2 matrix of floats
    public var import_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(import_)
    }

    /// return as 1 by 2 matrix of floats
    public var return_: MLMultiArray {
        provider.featureValue(for: "return")!.multiArrayValue!
    }

    /// return as 1 by 2 matrix of floats
    public var return_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(return_)
    }

    /// if as 1 by 2 matrix of floats
    public var if_: MLMultiArray {
        provider.featureValue(for: "if")!.multiArrayValue!
    }

    /// if as 1 by 2 matrix of floats
    public var if_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(if_)
    }

    /// for as 1 by 2 matrix of floats
    public var for_: MLMultiArray {
        provider.featureValue(for: "for")!.multiArrayValue!
    }

    /// for as 1 by 2 matrix of floats
    public var for_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(for_)
    }

    /// while as 1 by 2 matrix of floats
    public var while_: MLMultiArray {
        provider.featureValue(for: "while")!.multiArrayValue!
    }

    /// while as 1 by 2 matrix of floats
    public var while_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(while_)
    }

    /// struct as 1 by 2 matrix of floats
    public var struct_: MLMultiArray {
        provider.featureValue(for: "struct")!.multiArrayValue!
    }

    /// struct as 1 by 2 matrix of floats
    public var struct_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(struct_)
    }

    /// protocol as 1 by 2 matrix of floats
    public var protocol_: MLMultiArray {
        provider.featureValue(for: "protocol")!.multiArrayValue!
    }

    /// protocol as 1 by 2 matrix of floats
    public var protocol_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(protocol_)
    }

    /// model as 1 by 2 matrix of floats
    public var model: MLMultiArray {
        provider.featureValue(for: "model")!.multiArrayValue!
    }

    /// model as 1 by 2 matrix of floats
    public var modelShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(model)
    }

    /// input as 1 by 2 matrix of floats
    public var input: MLMultiArray {
        provider.featureValue(for: "input")!.multiArrayValue!
    }

    /// input as 1 by 2 matrix of floats
    public var inputShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(input)
    }

    /// output as 1 by 2 matrix of floats
    public var output: MLMultiArray {
        provider.featureValue(for: "output")!.multiArrayValue!
    }

    /// output as 1 by 2 matrix of floats
    public var outputShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(output)
    }

    /// features as 1 by 2 matrix of floats
    public var features: MLMultiArray {
        provider.featureValue(for: "features")!.multiArrayValue!
    }

    /// features as 1 by 2 matrix of floats
    public var featuresShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(features)
    }

    /// featureNames as 1 by 2 matrix of floats
    public var featureNames: MLMultiArray {
        provider.featureValue(for: "featureNames")!.multiArrayValue!
    }

    /// featureNames as 1 by 2 matrix of floats
    public var featureNamesShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(featureNames)
    }

    /// provider as 1 by 2 matrix of floats
    public var provider: MLMultiArray {
        provider.featureValue(for: "provider")!.multiArrayValue!
    }

    /// provider as 1 by 2 matrix of floats
    public var providerShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(provider)
    }

    /// id as 1 by 2 matrix of floats
    public var id_: MLMultiArray {
        provider.featureValue(for: "id")!.multiArrayValue!
    }

    /// id as 1 by 2 matrix of floats
    public var id_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(id_)
    }

    /// int as 1 by 2 matrix of floats
    public var int_: MLMultiArray {
        provider.featureValue(for: "int")!.multiArrayValue!
    }

    /// int as 1 by 2 matrix of floats
    public var int_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(int_)
    }

    /// float as 1 by 2 matrix of floats
    public var float_: MLMultiArray {
        provider.featureValue(for: "float")!.multiArrayValue!
    }

    /// float as 1 by 2 matrix of floats
    public var float_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(float_)
    }

    /// double as 1 by 2 matrix of floats
    public var double_: MLMultiArray {
        provider.featureValue(for: "double")!.multiArrayValue!
    }

    /// double as 1 by 2 matrix of floats
    public var double_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(double_)
    }

    /// char as 1 by 2 matrix of floats
    public var char_: MLMultiArray {
        provider.featureValue(for: "char")!.multiArrayValue!
    }

    /// char as 1 by 2 matrix of floats
    public var char_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(char_)
    }

    /// void as 1 by 2 matrix of floats
    public var void_: MLMultiArray {
        provider.featureValue(for: "void")!.multiArrayValue!
    }

    /// void as 1 by 2 matrix of floats
    public var void_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(void_)
    }

    /// bool_workaround as 1 by 2 matrix of floats
    public var bool_workaround: MLMultiArray {
        provider.featureValue(for: "bool_workaround")!.multiArrayValue!
    }

    /// bool_workaround as 1 by 2 matrix of floats
    public var bool_workaroundShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(bool_workaround)
    }

    /// BOOL as 1 by 2 matrix of floats
    public var BOOL_: MLMultiArray {
        provider.featureValue(for: "BOOL")!.multiArrayValue!
    }

    /// BOOL as 1 by 2 matrix of floats
    public var BOOL_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(BOOL_)
    }

    /// YES as 1 by 2 matrix of floats
    public var YES_: MLMultiArray {
        provider.featureValue(for: "YES")!.multiArrayValue!
    }

    /// YES as 1 by 2 matrix of floats
    public var YES_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(YES_)
    }

    /// NO as 1 by 2 matrix of floats
    public var NO_: MLMultiArray {
        provider.featureValue(for: "NO")!.multiArrayValue!
    }

    /// NO as 1 by 2 matrix of floats
    public var NO_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(NO_)
    }

    /// nil as 1 by 2 matrix of floats
    public var nil_: MLMultiArray {
        provider.featureValue(for: "nil")!.multiArrayValue!
    }

    /// nil as 1 by 2 matrix of floats
    public var nil_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(nil_)
    }

    /// new as 1 by 2 matrix of floats
    public var new: MLMultiArray {
        provider.featureValue(for: "new")!.multiArrayValue!
    }

    /// new as 1 by 2 matrix of floats
    public var newShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(new)
    }

    /// default as 1 by 2 matrix of floats
    public var default_: MLMultiArray {
        provider.featureValue(for: "default")!.multiArrayValue!
    }

    /// default as 1 by 2 matrix of floats
    public var default_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(default_)
    }

    /// switch as 1 by 2 matrix of floats
    public var switch_: MLMultiArray {
        provider.featureValue(for: "switch")!.multiArrayValue!
    }

    /// switch as 1 by 2 matrix of floats
    public var switch_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(switch_)
    }

    /// case as 1 by 2 matrix of floats
    public var case_: MLMultiArray {
        provider.featureValue(for: "case")!.multiArrayValue!
    }

    /// case as 1 by 2 matrix of floats
    public var case_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(case_)
    }

    /// operator as 1 by 2 matrix of floats
    public var operator_: MLMultiArray {
        provider.featureValue(for: "operator")!.multiArrayValue!
    }

    /// operator as 1 by 2 matrix of floats
    public var operator_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(operator_)
    }

    /// typealias as 1 by 2 matrix of floats
    public var typealias_: MLMultiArray {
        provider.featureValue(for: "typealias")!.multiArrayValue!
    }

    /// typealias as 1 by 2 matrix of floats
    public var typealias_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(typealias_)
    }

    /// extension as 1 by 2 matrix of floats
    public var extension_: MLMultiArray {
        provider.featureValue(for: "extension")!.multiArrayValue!
    }

    /// extension as 1 by 2 matrix of floats
    public var extension_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(extension_)
    }

    /// init as 1 by 2 matrix of floats
    public var init_: MLMultiArray {
        provider.featureValue(for: "init")!.multiArrayValue!
    }

    /// init as 1 by 2 matrix of floats
    public var init_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(init_)
    }

    /// deinit as 1 by 2 matrix of floats
    public var deinit_: MLMultiArray {
        provider.featureValue(for: "deinit")!.multiArrayValue!
    }

    /// deinit as 1 by 2 matrix of floats
    public var deinit_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(deinit_)
    }

    /// subscript as 1 by 2 matrix of floats
    public var subscript_: MLMultiArray {
        provider.featureValue(for: "subscript")!.multiArrayValue!
    }

    /// subscript as 1 by 2 matrix of floats
    public var subscript_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(subscript_)
    }

    /// static as 1 by 2 matrix of floats
    public var static_: MLMultiArray {
        provider.featureValue(for: "static")!.multiArrayValue!
    }

    /// static as 1 by 2 matrix of floats
    public var static_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(static_)
    }

    /// public as 1 by 2 matrix of floats
    public var public_: MLMultiArray {
        provider.featureValue(for: "public")!.multiArrayValue!
    }

    /// public as 1 by 2 matrix of floats
    public var public_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(public_)
    }

    /// private as 1 by 2 matrix of floats
    public var private_: MLMultiArray {
        provider.featureValue(for: "private")!.multiArrayValue!
    }

    /// private as 1 by 2 matrix of floats
    public var private_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(private_)
    }

    /// internal as 1 by 2 matrix of floats
    public var internal_: MLMultiArray {
        provider.featureValue(for: "internal")!.multiArrayValue!
    }

    /// internal as 1 by 2 matrix of floats
    public var internal_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(internal_)
    }

    /// true_workaround as 1 by 2 matrix of floats
    public var true_workaround: MLMultiArray {
        provider.featureValue(for: "true_workaround")!.multiArrayValue!
    }

    /// true_workaround as 1 by 2 matrix of floats
    public var true_workaroundShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(true_workaround)
    }

    /// false_workaround as 1 by 2 matrix of floats
    public var false_workaround: MLMultiArray {
        provider.featureValue(for: "false_workaround")!.multiArrayValue!
    }

    /// false_workaround as 1 by 2 matrix of floats
    public var false_workaroundShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(false_workaround)
    }

    /// Type as 1 by 2 matrix of floats
    public var Type_: MLMultiArray {
        provider.featureValue(for: "Type")!.multiArrayValue!
    }

    /// Type as 1 by 2 matrix of floats
    public var Type_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Type_)
    }

    /// Protocol as 1 by 2 matrix of floats
    public var Protocol_: MLMultiArray {
        provider.featureValue(for: "Protocol")!.multiArrayValue!
    }

    /// Protocol as 1 by 2 matrix of floats
    public var Protocol_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Protocol_)
    }

    /// super as 1 by 2 matrix of floats
    public var super_: MLMultiArray {
        provider.featureValue(for: "super")!.multiArrayValue!
    }

    /// super as 1 by 2 matrix of floats
    public var super_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(super_)
    }

    /// where as 1 by 2 matrix of floats
    public var where_: MLMultiArray {
        provider.featureValue(for: "where")!.multiArrayValue!
    }

    /// where as 1 by 2 matrix of floats
    public var where_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(where_)
    }

    /// as as 1 by 2 matrix of floats
    public var as_: MLMultiArray {
        provider.featureValue(for: "as")!.multiArrayValue!
    }

    /// as as 1 by 2 matrix of floats
    public var as_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(as_)
    }

    /// is as 1 by 2 matrix of floats
    public var is_: MLMultiArray {
        provider.featureValue(for: "is")!.multiArrayValue!
    }

    /// is as 1 by 2 matrix of floats
    public var is_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(is_)
    }

    /// try as 1 by 2 matrix of floats
    public var try: MLMultiArray {
        provider.featureValue(for: "try")!.multiArrayValue!
    }

    /// try as 1 by 2 matrix of floats
    public var tryShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(try)
    }

    /// throw as 1 by 2 matrix of floats
    public var throw: MLMultiArray {
        provider.featureValue(for: "throw")!.multiArrayValue!
    }

    /// throw as 1 by 2 matrix of floats
    public var throwShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(throw)
    }

    /// throws as 1 by 2 matrix of floats
    public var throws: MLMultiArray {
        provider.featureValue(for: "throws")!.multiArrayValue!
    }

    /// throws as 1 by 2 matrix of floats
    public var throwsShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(throws)
    }

    /// rethrows as 1 by 2 matrix of floats
    public var rethrows: MLMultiArray {
        provider.featureValue(for: "rethrows")!.multiArrayValue!
    }

    /// rethrows as 1 by 2 matrix of floats
    public var rethrowsShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(rethrows)
    }

    /// async as 1 by 2 matrix of floats
    public var async: MLMultiArray {
        provider.featureValue(for: "async")!.multiArrayValue!
    }

    /// async as 1 by 2 matrix of floats
    public var asyncShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(async)
    }

    /// await as 1 by 2 matrix of floats
    public var await: MLMultiArray {
        provider.featureValue(for: "await")!.multiArrayValue!
    }

    /// await as 1 by 2 matrix of floats
    public var awaitShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(await)
    }

    /// defer as 1 by 2 matrix of floats
    public var defer: MLMultiArray {
        provider.featureValue(for: "defer")!.multiArrayValue!
    }

    /// defer as 1 by 2 matrix of floats
    public var deferShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(defer)
    }

    /// guard as 1 by 2 matrix of floats
    public var guard: MLMultiArray {
        provider.featureValue(for: "guard")!.multiArrayValue!
    }

    /// guard as 1 by 2 matrix of floats
    public var guardShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(guard)
    }

    /// repeat as 1 by 2 matrix of floats
    public var repeat: MLMultiArray {
        provider.featureValue(for: "repeat")!.multiArrayValue!
    }

    /// repeat as 1 by 2 matrix of floats
    public var repeatShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(repeat)
    }

    /// break as 1 by 2 matrix of floats
    public var break_: MLMultiArray {
        provider.featureValue(for: "break")!.multiArrayValue!
    }

    /// break as 1 by 2 matrix of floats
    public var break_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(break_)
    }

    /// continue as 1 by 2 matrix of floats
    public var continue_: MLMultiArray {
        provider.featureValue(for: "continue")!.multiArrayValue!
    }

    /// continue as 1 by 2 matrix of floats
    public var continue_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(continue_)
    }

    /// fallthrough as 1 by 2 matrix of floats
    public var fallthrough_: MLMultiArray {
        provider.featureValue(for: "fallthrough")!.multiArrayValue!
    }

    /// fallthrough as 1 by 2 matrix of floats
    public var fallthrough_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(fallthrough_)
    }

    /// associatedtype as 1 by 2 matrix of floats
    public var associatedtype: MLMultiArray {
        provider.featureValue(for: "associatedtype")!.multiArrayValue!
    }

    /// associatedtype as 1 by 2 matrix of floats
    public var associatedtypeShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(associatedtype)
    }

    /// enum as 1 by 2 matrix of floats
    public var enum_: MLMultiArray {
        provider.featureValue(for: "enum")!.multiArrayValue!
    }

    /// enum as 1 by 2 matrix of floats
    public var enum_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(enum_)
    }

    /// union as 1 by 2 matrix of floats
    public var union_: MLMultiArray {
        provider.featureValue(for: "union")!.multiArrayValue!
    }

    /// union as 1 by 2 matrix of floats
    public var union_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(union_)
    }

    /// const as 1 by 2 matrix of floats
    public var const_: MLMultiArray {
        provider.featureValue(for: "const")!.multiArrayValue!
    }

    /// const as 1 by 2 matrix of floats
    public var const_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(const_)
    }

    /// signed as 1 by 2 matrix of floats
    public var signed_: MLMultiArray {
        provider.featureValue(for: "signed")!.multiArrayValue!
    }

    /// signed as 1 by 2 matrix of floats
    public var signed_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(signed_)
    }

    /// unsigned as 1 by 2 matrix of floats
    public var unsigned_: MLMultiArray {
        provider.featureValue(for: "unsigned")!.multiArrayValue!
    }

    /// unsigned as 1 by 2 matrix of floats
    public var unsigned_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(unsigned_)
    }

    /// long as 1 by 2 matrix of floats
    public var long_: MLMultiArray {
        provider.featureValue(for: "long")!.multiArrayValue!
    }

    /// long as 1 by 2 matrix of floats
    public var long_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(long_)
    }

    /// short as 1 by 2 matrix of floats
    public var short_: MLMultiArray {
        provider.featureValue(for: "short")!.multiArrayValue!
    }

    /// short as 1 by 2 matrix of floats
    public var short_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(short_)
    }

    /// auto as 1 by 2 matrix of floats
    public var auto_: MLMultiArray {
        provider.featureValue(for: "auto")!.multiArrayValue!
    }

    /// auto as 1 by 2 matrix of floats
    public var auto_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(auto_)
    }

    /// register as 1 by 2 matrix of floats
    public var register_: MLMultiArray {
        provider.featureValue(for: "register")!.multiArrayValue!
    }

    /// register as 1 by 2 matrix of floats
    public var register_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(register_)
    }

    /// extern as 1 by 2 matrix of floats
    public var extern_: MLMultiArray {
        provider.featureValue(for: "extern")!.multiArrayValue!
    }

    /// extern as 1 by 2 matrix of floats
    public var extern_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(extern_)
    }

    /// volatile as 1 by 2 matrix of floats
    public var volatile_: MLMultiArray {
        provider.featureValue(for: "volatile")!.multiArrayValue!
    }

    /// volatile as 1 by 2 matrix of floats
    public var volatile_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(volatile_)
    }

    /// goto as 1 by 2 matrix of floats
    public var goto_: MLMultiArray {
        provider.featureValue(for: "goto")!.multiArrayValue!
    }

    /// goto as 1 by 2 matrix of floats
    public var goto_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(goto_)
    }

    /// sizeof as 1 by 2 matrix of floats
    public var sizeof_: MLMultiArray {
        provider.featureValue(for: "sizeof")!.multiArrayValue!
    }

    /// sizeof as 1 by 2 matrix of floats
    public var sizeof_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(sizeof_)
    }

    /// typedef as 1 by 2 matrix of floats
    public var typedef_: MLMultiArray {
        provider.featureValue(for: "typedef")!.multiArrayValue!
    }

    /// typedef as 1 by 2 matrix of floats
    public var typedef_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(typedef_)
    }

    /// oneway as 1 by 2 matrix of floats
    public var oneway_: MLMultiArray {
        provider.featureValue(for: "oneway")!.multiArrayValue!
    }

    /// oneway as 1 by 2 matrix of floats
    public var oneway_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(oneway_)
    }

    /// bycopy as 1 by 2 matrix of floats
    public var bycopy_: MLMultiArray {
        provider.featureValue(for: "bycopy")!.multiArrayValue!
    }

    /// bycopy as 1 by 2 matrix of floats
    public var bycopy_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(bycopy_)
    }

    /// byref as 1 by 2 matrix of floats
    public var byref_: MLMultiArray {
        provider.featureValue(for: "byref")!.multiArrayValue!
    }

    /// byref as 1 by 2 matrix of floats
    public var byref_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(byref_)
    }

    /// restrict as 1 by 2 matrix of floats
    public var restrict_: MLMultiArray {
        provider.featureValue(for: "restrict")!.multiArrayValue!
    }

    /// restrict as 1 by 2 matrix of floats
    public var restrict_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(restrict_)
    }

    /// inline as 1 by 2 matrix of floats
    public var inline_: MLMultiArray {
        provider.featureValue(for: "inline")!.multiArrayValue!
    }

    /// inline as 1 by 2 matrix of floats
    public var inline_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(inline_)
    }

    /// template as 1 by 2 matrix of floats
    public var template: MLMultiArray {
        provider.featureValue(for: "template")!.multiArrayValue!
    }

    /// template as 1 by 2 matrix of floats
    public var templateShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(template)
    }

    /// namespace as 1 by 2 matrix of floats
    public var namespace: MLMultiArray {
        provider.featureValue(for: "namespace")!.multiArrayValue!
    }

    /// namespace as 1 by 2 matrix of floats
    public var namespaceShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(namespace)
    }

    /// delete as 1 by 2 matrix of floats
    public var delete: MLMultiArray {
        provider.featureValue(for: "delete")!.multiArrayValue!
    }

    /// delete as 1 by 2 matrix of floats
    public var deleteShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(delete)
    }

    /// this as 1 by 2 matrix of floats
    public var this: MLMultiArray {
        provider.featureValue(for: "this")!.multiArrayValue!
    }

    /// this as 1 by 2 matrix of floats
    public var thisShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(this)
    }

    /// friend as 1 by 2 matrix of floats
    public var friend: MLMultiArray {
        provider.featureValue(for: "friend")!.multiArrayValue!
    }

    /// friend as 1 by 2 matrix of floats
    public var friendShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(friend)
    }

    /// virtual as 1 by 2 matrix of floats
    public var virtual: MLMultiArray {
        provider.featureValue(for: "virtual")!.multiArrayValue!
    }

    /// virtual as 1 by 2 matrix of floats
    public var virtualShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(virtual)
    }

    /// mutable as 1 by 2 matrix of floats
    public var mutable: MLMultiArray {
        provider.featureValue(for: "mutable")!.multiArrayValue!
    }

    /// mutable as 1 by 2 matrix of floats
    public var mutableShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(mutable)
    }

    /// explicit as 1 by 2 matrix of floats
    public var explicit: MLMultiArray {
        provider.featureValue(for: "explicit")!.multiArrayValue!
    }

    /// explicit as 1 by 2 matrix of floats
    public var explicitShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(explicit)
    }

    /// export as 1 by 2 matrix of floats
    public var export: MLMultiArray {
        provider.featureValue(for: "export")!.multiArrayValue!
    }

    /// export as 1 by 2 matrix of floats
    public var exportShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(export)
    }

    /// Any as 1 by 2 matrix of floats
    public var Any: MLMultiArray {
        provider.featureValue(for: "Any")!.multiArrayValue!
    }

    /// Any as 1 by 2 matrix of floats
    public var AnyShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Any)
    }

    /// Self as 1 by 2 matrix of floats
    public var Self_: MLMultiArray {
        provider.featureValue(for: "Self")!.multiArrayValue!
    }

    /// Self as 1 by 2 matrix of floats
    public var Self_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Self_)
    }

    /// some as 1 by 2 matrix of floats
    public var some: MLMultiArray {
        provider.featureValue(for: "some")!.multiArrayValue!
    }

    /// some as 1 by 2 matrix of floats
    public var someShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(some)
    }

    /// any_workaround as 1 by 2 matrix of floats
    public var any_workaround: MLMultiArray {
        provider.featureValue(for: "any_workaround")!.multiArrayValue!
    }

    /// any_workaround as 1 by 2 matrix of floats
    public var any_workaroundShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(any_workaround)
    }

    /// do as 1 by 2 matrix of floats
    public var do_: MLMultiArray {
        provider.featureValue(for: "do")!.multiArrayValue!
    }

    /// do as 1 by 2 matrix of floats
    public var do_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(do_)
    }

    /// else as 1 by 2 matrix of floats
    public var else_: MLMultiArray {
        provider.featureValue(for: "else")!.multiArrayValue!
    }

    /// else as 1 by 2 matrix of floats
    public var else_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(else_)
    }

    /// open as 1 by 2 matrix of floats
    public var open: MLMultiArray {
        provider.featureValue(for: "open")!.multiArrayValue!
    }

    /// open as 1 by 2 matrix of floats
    public var openShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(open)
    }

    /// fileprivate as 1 by 2 matrix of floats
    public var fileprivate: MLMultiArray {
        provider.featureValue(for: "fileprivate")!.multiArrayValue!
    }

    /// fileprivate as 1 by 2 matrix of floats
    public var fileprivateShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(fileprivate)
    }

    /// convenience as 1 by 2 matrix of floats
    public var convenience_: MLMultiArray {
        provider.featureValue(for: "convenience")!.multiArrayValue!
    }

    /// convenience as 1 by 2 matrix of floats
    public var convenience_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(convenience_)
    }

    /// dynamic as 1 by 2 matrix of floats
    public var dynamic_: MLMultiArray {
        provider.featureValue(for: "dynamic")!.multiArrayValue!
    }

    /// dynamic as 1 by 2 matrix of floats
    public var dynamic_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(dynamic_)
    }

    /// final as 1 by 2 matrix of floats
    public var final_: MLMultiArray {
        provider.featureValue(for: "final")!.multiArrayValue!
    }

    /// final as 1 by 2 matrix of floats
    public var final_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(final_)
    }

    /// lazy as 1 by 2 matrix of floats
    public var lazy_: MLMultiArray {
        provider.featureValue(for: "lazy")!.multiArrayValue!
    }

    /// lazy as 1 by 2 matrix of floats
    public var lazy_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(lazy_)
    }

    /// optional as 1 by 2 matrix of floats
    public var optional_: MLMultiArray {
        provider.featureValue(for: "optional")!.multiArrayValue!
    }

    /// optional as 1 by 2 matrix of floats
    public var optional_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(optional_)
    }

    /// required as 1 by 2 matrix of floats
    public var required_: MLMultiArray {
        provider.featureValue(for: "required")!.multiArrayValue!
    }

    /// required as 1 by 2 matrix of floats
    public var required_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(required_)
    }

    /// override as 1 by 2 matrix of floats
    public var override_: MLMultiArray {
        provider.featureValue(for: "override")!.multiArrayValue!
    }

    /// override as 1 by 2 matrix of floats
    public var override_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(override_)
    }

    /// weak as 1 by 2 matrix of floats
    public var weak_: MLMultiArray {
        provider.featureValue(for: "weak")!.multiArrayValue!
    }

    /// weak as 1 by 2 matrix of floats
    public var weak_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(weak_)
    }

    /// unowned as 1 by 2 matrix of floats
    public var unowned_: MLMultiArray {
        provider.featureValue(for: "unowned")!.multiArrayValue!
    }

    /// unowned as 1 by 2 matrix of floats
    public var unowned_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(unowned_)
    }

    /// mutating as 1 by 2 matrix of floats
    public var mutating_: MLMultiArray {
        provider.featureValue(for: "mutating")!.multiArrayValue!
    }

    /// mutating as 1 by 2 matrix of floats
    public var mutating_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(mutating_)
    }

    /// nonmutating as 1 by 2 matrix of floats
    public var nonmutating_: MLMultiArray {
        provider.featureValue(for: "nonmutating")!.multiArrayValue!
    }

    /// nonmutating as 1 by 2 matrix of floats
    public var nonmutating_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(nonmutating_)
    }

    /// indirect as 1 by 2 matrix of floats
    public var indirect: MLMultiArray {
        provider.featureValue(for: "indirect")!.multiArrayValue!
    }

    /// indirect as 1 by 2 matrix of floats
    public var indirectShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(indirect)
    }

    /// prefix as 1 by 2 matrix of floats
    public var prefix_: MLMultiArray {
        provider.featureValue(for: "prefix")!.multiArrayValue!
    }

    /// prefix as 1 by 2 matrix of floats
    public var prefix_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(prefix_)
    }

    /// postfix as 1 by 2 matrix of floats
    public var postfix_: MLMultiArray {
        provider.featureValue(for: "postfix")!.multiArrayValue!
    }

    /// postfix as 1 by 2 matrix of floats
    public var postfix_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(postfix_)
    }

    /// infix as 1 by 2 matrix of floats
    public var infix_: MLMultiArray {
        provider.featureValue(for: "infix")!.multiArrayValue!
    }

    /// infix as 1 by 2 matrix of floats
    public var infix_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(infix_)
    }

    /// get as 1 by 2 matrix of floats
    public var get_: MLMultiArray {
        provider.featureValue(for: "get")!.multiArrayValue!
    }

    /// get as 1 by 2 matrix of floats
    public var get_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(get_)
    }

    /// set as 1 by 2 matrix of floats
    public var set_: MLMultiArray {
        provider.featureValue(for: "set")!.multiArrayValue!
    }

    /// set as 1 by 2 matrix of floats
    public var set_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(set_)
    }

    /// willSet as 1 by 2 matrix of floats
    public var willSet: MLMultiArray {
        provider.featureValue(for: "willSet")!.multiArrayValue!
    }

    /// willSet as 1 by 2 matrix of floats
    public var willSetShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(willSet)
    }

    /// didSet as 1 by 2 matrix of floats
    public var didSet_: MLMultiArray {
        provider.featureValue(for: "didSet")!.multiArrayValue!
    }

    /// didSet as 1 by 2 matrix of floats
    public var didSet_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(didSet_)
    }

    /// NULL as 1 by 2 matrix of floats
    public var NULL_: MLMultiArray {
        provider.featureValue(for: "NULL")!.multiArrayValue!
    }

    /// NULL as 1 by 2 matrix of floats
    public var NULL_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(NULL_)
    }

    /// Class as 1 by 2 matrix of floats
    public var Class_: MLMultiArray {
        provider.featureValue(for: "Class")!.multiArrayValue!
    }

    /// Class as 1 by 2 matrix of floats
    public var Class_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Class_)
    }

    /// SEL as 1 by 2 matrix of floats
    public var SEL_: MLMultiArray {
        provider.featureValue(for: "SEL")!.multiArrayValue!
    }

    /// SEL as 1 by 2 matrix of floats
    public var SEL_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(SEL_)
    }

    /// IMP as 1 by 2 matrix of floats
    public var IMP_: MLMultiArray {
        provider.featureValue(for: "IMP")!.multiArrayValue!
    }

    /// IMP as 1 by 2 matrix of floats
    public var IMP_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(IMP_)
    }

    /// nonatomic as 1 by 2 matrix of floats
    public var nonatomic_: MLMultiArray {
        provider.featureValue(for: "nonatomic")!.multiArrayValue!
    }

    /// nonatomic as 1 by 2 matrix of floats
    public var nonatomic_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(nonatomic_)
    }

    /// atomic as 1 by 2 matrix of floats
    public var atomic_: MLMultiArray {
        provider.featureValue(for: "atomic")!.multiArrayValue!
    }

    /// atomic as 1 by 2 matrix of floats
    public var atomic_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(atomic_)
    }

    /// strong as 1 by 2 matrix of floats
    public var strong: MLMultiArray {
        provider.featureValue(for: "strong")!.multiArrayValue!
    }

    /// strong as 1 by 2 matrix of floats
    public var strongShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(strong)
    }

    /// copy as 1 by 2 matrix of floats
    public var copy: MLMultiArray {
        provider.featureValue(for: "copy")!.multiArrayValue!
    }

    /// copy as 1 by 2 matrix of floats
    public var copyShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(copy)
    }

    /// assign as 1 by 2 matrix of floats
    public var assign: MLMultiArray {
        provider.featureValue(for: "assign")!.multiArrayValue!
    }

    /// assign as 1 by 2 matrix of floats
    public var assignShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(assign)
    }

    /// readonly as 1 by 2 matrix of floats
    public var readonly: MLMultiArray {
        provider.featureValue(for: "readonly")!.multiArrayValue!
    }

    /// readonly as 1 by 2 matrix of floats
    public var readonlyShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(readonly)
    }

    /// readwrite as 1 by 2 matrix of floats
    public var readwrite: MLMultiArray {
        provider.featureValue(for: "readwrite")!.multiArrayValue!
    }

    /// readwrite as 1 by 2 matrix of floats
    public var readwriteShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(readwrite)
    }

    /// instancetype as 1 by 2 matrix of floats
    public var instancetype: MLMultiArray {
        provider.featureValue(for: "instancetype")!.multiArrayValue!
    }

    /// instancetype as 1 by 2 matrix of floats
    public var instancetypeShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(instancetype)
    }

    /// description as 1 by 2 matrix of floats
    public var description: MLMultiArray {
        provider.featureValue(for: "description")!.multiArrayValue!
    }

    /// description as 1 by 2 matrix of floats
    public var descriptionShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(description)
    }

    /// hash as 1 by 2 matrix of floats
    public var hash: MLMultiArray {
        provider.featureValue(for: "hash")!.multiArrayValue!
    }

    /// hash as 1 by 2 matrix of floats
    public var hashShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(hash)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(out_: MLMultiArray, var_: MLMultiArray, in_: MLMultiArray, inout_: MLMultiArray, self_: MLMultiArray, class_: MLMultiArray, func_workaround: MLMultiArray, let_: MLMultiArray, import_: MLMultiArray, return_: MLMultiArray, if_: MLMultiArray, for_: MLMultiArray, while_: MLMultiArray, struct_: MLMultiArray, protocol_: MLMultiArray, model: MLMultiArray, input: MLMultiArray, output: MLMultiArray, features: MLMultiArray, featureNames: MLMultiArray, provider: MLMultiArray, id_: MLMultiArray, int_: MLMultiArray, float_: MLMultiArray, double_: MLMultiArray, char_: MLMultiArray, void_: MLMultiArray, bool_workaround: MLMultiArray, BOOL_: MLMultiArray, YES_: MLMultiArray, NO_: MLMultiArray, nil_: MLMultiArray, new: MLMultiArray, default_: MLMultiArray, switch_: MLMultiArray, case_: MLMultiArray, operator_: MLMultiArray, typealias_: MLMultiArray, extension_: MLMultiArray, init_: MLMultiArray, deinit_: MLMultiArray, subscript_: MLMultiArray, static_: MLMultiArray, public_: MLMultiArray, private_: MLMultiArray, internal_: MLMultiArray, true_workaround: MLMultiArray, false_workaround: MLMultiArray, Type_: MLMultiArray, Protocol_: MLMultiArray, super_: MLMultiArray, where_: MLMultiArray, as_: MLMultiArray, is_: MLMultiArray, try: MLMultiArray, throw: MLMultiArray, throws: MLMultiArray, rethrows: MLMultiArray, async: MLMultiArray, await: MLMultiArray, defer: MLMultiArray, guard: MLMultiArray, repeat: MLMultiArray, break_: MLMultiArray, continue_: MLMultiArray, fallthrough_: MLMultiArray, associatedtype: MLMultiArray, enum_: MLMultiArray, union_: MLMultiArray, const_: MLMultiArray, signed_: MLMultiArray, unsigned_: MLMultiArray, long_: MLMultiArray, short_: MLMultiArray, auto_: MLMultiArray, register_: MLMultiArray, extern_: MLMultiArray, volatile_: MLMultiArray, goto_: MLMultiArray, sizeof_: MLMultiArray, typedef_: MLMultiArray, oneway_: MLMultiArray, bycopy_: MLMultiArray, byref_: MLMultiArray, restrict_: MLMultiArray, inline_: MLMultiArray, template: MLMultiArray, namespace: MLMultiArray, delete: MLMultiArray, this: MLMultiArray, friend: MLMultiArray, virtual: MLMultiArray, mutable: MLMultiArray, explicit: MLMultiArray, export: MLMultiArray, Any: MLMultiArray, Self_: MLMultiArray, some: MLMultiArray, any_workaround: MLMultiArray, do_: MLMultiArray, else_: MLMultiArray, open: MLMultiArray, fileprivate: MLMultiArray, convenience_: MLMultiArray, dynamic_: MLMultiArray, final_: MLMultiArray, lazy_: MLMultiArray, optional_: MLMultiArray, required_: MLMultiArray, override_: MLMultiArray, weak_: MLMultiArray, unowned_: MLMultiArray, mutating_: MLMultiArray, nonmutating_: MLMultiArray, indirect: MLMultiArray, prefix_: MLMultiArray, postfix_: MLMultiArray, infix_: MLMultiArray, get_: MLMultiArray, set_: MLMultiArray, willSet: MLMultiArray, didSet_: MLMultiArray, NULL_: MLMultiArray, Class_: MLMultiArray, SEL_: MLMultiArray, IMP_: MLMultiArray, nonatomic_: MLMultiArray, atomic_: MLMultiArray, strong: MLMultiArray, copy: MLMultiArray, assign: MLMultiArray, readonly: MLMultiArray, readwrite: MLMultiArray, instancetype: MLMultiArray, description: MLMultiArray, hash: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["out" : MLFeatureValue(multiArray: out_), "var" : MLFeatureValue(multiArray: var_), "in" : MLFeatureValue(multiArray: in_), "inout" : MLFeatureValue(multiArray: inout_), "self" : MLFeatureValue(multiArray: self_), "class" : MLFeatureValue(multiArray: class_), "func_workaround" : MLFeatureValue(multiArray: func_workaround), "let" : MLFeatureValue(multiArray: let_), "import" : MLFeatureValue(multiArray: import_), "return" : MLFeatureValue(multiArray: return_), "if" : MLFeatureValue(multiArray: if_), "for" : MLFeatureValue(multiArray: for_), "while" : MLFeatureValue(multiArray: while_), "struct" : MLFeatureValue(multiArray: struct_), "protocol" : MLFeatureValue(multiArray: protocol_), "model" : MLFeatureValue(multiArray: model), "input" : MLFeatureValue(multiArray: input), "output" : MLFeatureValue(multiArray: output), "features" : MLFeatureValue(multiArray: features), "featureNames" : MLFeatureValue(multiArray: featureNames), "provider" : MLFeatureValue(multiArray: provider), "id" : MLFeatureValue(multiArray: id_), "int" : MLFeatureValue(multiArray: int_), "float" : MLFeatureValue(multiArray: float_), "double" : MLFeatureValue(multiArray: double_), "char" : MLFeatureValue(multiArray: char_), "void" : MLFeatureValue(multiArray: void_), "bool_workaround" : MLFeatureValue(multiArray: bool_workaround), "BOOL" : MLFeatureValue(multiArray: BOOL_), "YES" : MLFeatureValue(multiArray: YES_), "NO" : MLFeatureValue(multiArray: NO_), "nil" : MLFeatureValue(multiArray: nil_), "new" : MLFeatureValue(multiArray: new), "default" : MLFeatureValue(multiArray: default_), "switch" : MLFeatureValue(multiArray: switch_), "case" : MLFeatureValue(multiArray: case_), "operator" : MLFeatureValue(multiArray: operator_), "typealias" : MLFeatureValue(multiArray: typealias_), "extension" : MLFeatureValue(multiArray: extension_), "init" : MLFeatureValue(multiArray: init_), "deinit" : MLFeatureValue(multiArray: deinit_), "subscript" : MLFeatureValue(multiArray: subscript_), "static" : MLFeatureValue(multiArray: static_), "public" : MLFeatureValue(multiArray: public_), "private" : MLFeatureValue(multiArray: private_), "internal" : MLFeatureValue(multiArray: internal_), "true_workaround" : MLFeatureValue(multiArray: true_workaround), "false_workaround" : MLFeatureValue(multiArray: false_workaround), "Type" : MLFeatureValue(multiArray: Type_), "Protocol" : MLFeatureValue(multiArray: Protocol_), "super" : MLFeatureValue(multiArray: super_), "where" : MLFeatureValue(multiArray: where_), "as" : MLFeatureValue(multiArray: as_), "is" : MLFeatureValue(multiArray: is_), "try" : MLFeatureValue(multiArray: try), "throw" : MLFeatureValue(multiArray: throw), "throws" : MLFeatureValue(multiArray: throws), "rethrows" : MLFeatureValue(multiArray: rethrows), "async" : MLFeatureValue(multiArray: async), "await" : MLFeatureValue(multiArray: await), "defer" : MLFeatureValue(multiArray: defer), "guard" : MLFeatureValue(multiArray: guard), "repeat" : MLFeatureValue(multiArray: repeat), "break" : MLFeatureValue(multiArray: break_), "continue" : MLFeatureValue(multiArray: continue_), "fallthrough" : MLFeatureValue(multiArray: fallthrough_), "associatedtype" : MLFeatureValue(multiArray: associatedtype), "enum" : MLFeatureValue(multiArray: enum_), "union" : MLFeatureValue(multiArray: union_), "const" : MLFeatureValue(multiArray: const_), "signed" : MLFeatureValue(multiArray: signed_), "unsigned" : MLFeatureValue(multiArray: unsigned_), "long" : MLFeatureValue(multiArray: long_), "short" : MLFeatureValue(multiArray: short_), "auto" : MLFeatureValue(multiArray: auto_), "register" : MLFeatureValue(multiArray: register_), "extern" : MLFeatureValue(multiArray: extern_), "volatile" : MLFeatureValue(multiArray: volatile_), "goto" : MLFeatureValue(multiArray: goto_), "sizeof" : MLFeatureValue(multiArray: sizeof_), "typedef" : MLFeatureValue(multiArray: typedef_), "oneway" : MLFeatureValue(multiArray: oneway_), "bycopy" : MLFeatureValue(multiArray: bycopy_), "byref" : MLFeatureValue(multiArray: byref_), "restrict" : MLFeatureValue(multiArray: restrict_), "inline" : MLFeatureValue(multiArray: inline_), "template" : MLFeatureValue(multiArray: template), "namespace" : MLFeatureValue(multiArray: namespace), "delete" : MLFeatureValue(multiArray: delete), "this" : MLFeatureValue(multiArray: this), "friend" : MLFeatureValue(multiArray: friend), "virtual" : MLFeatureValue(multiArray: virtual), "mutable" : MLFeatureValue(multiArray: mutable), "explicit" : MLFeatureValue(multiArray: explicit), "export" : MLFeatureValue(multiArray: export), "Any" : MLFeatureValue(multiArray: Any), "Self" : MLFeatureValue(multiArray: Self_), "some" : MLFeatureValue(multiArray: some), "any_workaround" : MLFeatureValue(multiArray: any_workaround), "do" : MLFeatureValue(multiArray: do_), "else" : MLFeatureValue(multiArray: else_), "open" : MLFeatureValue(multiArray: open), "fileprivate" : MLFeatureValue(multiArray: fileprivate), "convenience" : MLFeatureValue(multiArray: convenience_), "dynamic" : MLFeatureValue(multiArray: dynamic_), "final" : MLFeatureValue(multiArray: final_), "lazy" : MLFeatureValue(multiArray: lazy_), "optional" : MLFeatureValue(multiArray: optional_), "required" : MLFeatureValue(multiArray: required_), "override" : MLFeatureValue(multiArray: override_), "weak" : MLFeatureValue(multiArray: weak_), "unowned" : MLFeatureValue(multiArray: unowned_), "mutating" : MLFeatureValue(multiArray: mutating_), "nonmutating" : MLFeatureValue(multiArray: nonmutating_), "indirect" : MLFeatureValue(multiArray: indirect), "prefix" : MLFeatureValue(multiArray: prefix_), "postfix" : MLFeatureValue(multiArray: postfix_), "infix" : MLFeatureValue(multiArray: infix_), "get" : MLFeatureValue(multiArray: get_), "set" : MLFeatureValue(multiArray: set_), "willSet" : MLFeatureValue(multiArray: willSet), "didSet" : MLFeatureValue(multiArray: didSet_), "NULL" : MLFeatureValue(multiArray: NULL_), "Class" : MLFeatureValue(multiArray: Class_), "SEL" : MLFeatureValue(multiArray: SEL_), "IMP" : MLFeatureValue(multiArray: IMP_), "nonatomic" : MLFeatureValue(multiArray: nonatomic_), "atomic" : MLFeatureValue(multiArray: atomic_), "strong" : MLFeatureValue(multiArray: strong), "copy" : MLFeatureValue(multiArray: copy), "assign" : MLFeatureValue(multiArray: assign), "readonly" : MLFeatureValue(multiArray: readonly), "readwrite" : MLFeatureValue(multiArray: readwrite), "instancetype" : MLFeatureValue(multiArray: instancetype), "description" : MLFeatureValue(multiArray: description), "hash" : MLFeatureValue(multiArray: hash)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class Names {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Names", withExtension:"mlmodelc")!
    }

    /**
        Construct Names instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Names.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Names.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct a model with configuration

        - parameters:
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(configuration: MLModelConfiguration = MLModelConfiguration()) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Names instance with explicit path to mlmodelc file
        - parameters:
           - modelURL: the file url of the model

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL) throws {
        try self.init(model: MLModel(contentsOf: modelURL))
    }

    /**
        Construct a model with URL of the .mlmodelc directory and configuration

        - parameters:
           - modelURL: the file url of the model
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {
        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))
    }

    /**
        Construct Names instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Names, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Names instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Names {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Names instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Names, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Names(model: model)))
            }
        }
    }

    /**
        Construct Names instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Names {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Names(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as NamesInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as NamesOutput
    */
    public func prediction(input: NamesInput) throws -> NamesOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as NamesInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as NamesOutput
    */
    public func prediction(input: NamesInput, options: MLPredictionOptions) throws -> NamesOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return NamesOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as NamesInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as NamesOutput
    */
    public func prediction(input: NamesInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> NamesOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return NamesOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 by 2 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as NamesOutput
    */
    public func prediction(x: MLMultiArray) throws -> NamesOutput {
        let input_ = NamesInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 by 2 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as NamesOutput
    */

    public func prediction(x: MLShapedArray<Float>) throws -> NamesOutput {
        let input_ = NamesInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [NamesInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [NamesOutput]
    */
    public func predictions(inputs: [NamesInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [NamesOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [NamesOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  NamesOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
