//
// Names2.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class Names2Input : MLFeatureProvider {

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
public class Names2Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// catch as 1 by 2 matrix of floats
    public var catch: MLMultiArray {
        provider.featureValue(for: "catch")!.multiArrayValue!
    }

    /// catch as 1 by 2 matrix of floats
    public var catchShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(catch)
    }

    /// package as 1 by 2 matrix of floats
    public var package: MLMultiArray {
        provider.featureValue(for: "package")!.multiArrayValue!
    }

    /// package as 1 by 2 matrix of floats
    public var packageShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(package)
    }

    /// macro as 1 by 2 matrix of floats
    public var macro: MLMultiArray {
        provider.featureValue(for: "macro")!.multiArrayValue!
    }

    /// macro as 1 by 2 matrix of floats
    public var macroShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(macro)
    }

    /// borrowing as 1 by 2 matrix of floats
    public var borrowing: MLMultiArray {
        provider.featureValue(for: "borrowing")!.multiArrayValue!
    }

    /// borrowing as 1 by 2 matrix of floats
    public var borrowingShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(borrowing)
    }

    /// consuming as 1 by 2 matrix of floats
    public var consuming: MLMultiArray {
        provider.featureValue(for: "consuming")!.multiArrayValue!
    }

    /// consuming as 1 by 2 matrix of floats
    public var consumingShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(consuming)
    }

    /// nonisolated as 1 by 2 matrix of floats
    public var nonisolated: MLMultiArray {
        provider.featureValue(for: "nonisolated")!.multiArrayValue!
    }

    /// nonisolated as 1 by 2 matrix of floats
    public var nonisolatedShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(nonisolated)
    }

    /// isolated as 1 by 2 matrix of floats
    public var isolated: MLMultiArray {
        provider.featureValue(for: "isolated")!.multiArrayValue!
    }

    /// isolated as 1 by 2 matrix of floats
    public var isolatedShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(isolated)
    }

    /// actor as 1 by 2 matrix of floats
    public var actor: MLMultiArray {
        provider.featureValue(for: "actor")!.multiArrayValue!
    }

    /// actor as 1 by 2 matrix of floats
    public var actorShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(actor)
    }

    /// each as 1 by 2 matrix of floats
    public var each: MLMultiArray {
        provider.featureValue(for: "each")!.multiArrayValue!
    }

    /// each as 1 by 2 matrix of floats
    public var eachShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(each)
    }

    /// asm as 1 by 2 matrix of floats
    public var asm: MLMultiArray {
        provider.featureValue(for: "asm")!.multiArrayValue!
    }

    /// asm as 1 by 2 matrix of floats
    public var asmShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(asm)
    }

    /// typeof as 1 by 2 matrix of floats
    public var typeof: MLMultiArray {
        provider.featureValue(for: "typeof")!.multiArrayValue!
    }

    /// typeof as 1 by 2 matrix of floats
    public var typeofShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(typeof)
    }

    /// nullable as 1 by 2 matrix of floats
    public var nullable: MLMultiArray {
        provider.featureValue(for: "nullable")!.multiArrayValue!
    }

    /// nullable as 1 by 2 matrix of floats
    public var nullableShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(nullable)
    }

    /// nonnull as 1 by 2 matrix of floats
    public var nonnull: MLMultiArray {
        provider.featureValue(for: "nonnull")!.multiArrayValue!
    }

    /// nonnull as 1 by 2 matrix of floats
    public var nonnullShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(nonnull)
    }

    /// property as 1 by 2 matrix of floats
    public var property: MLMultiArray {
        provider.featureValue(for: "property")!.multiArrayValue!
    }

    /// property as 1 by 2 matrix of floats
    public var propertyShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(property)
    }

    /// synthesize as 1 by 2 matrix of floats
    public var synthesize: MLMultiArray {
        provider.featureValue(for: "synthesize")!.multiArrayValue!
    }

    /// synthesize as 1 by 2 matrix of floats
    public var synthesizeShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(synthesize)
    }

    /// interface as 1 by 2 matrix of floats
    public var interface: MLMultiArray {
        provider.featureValue(for: "interface")!.multiArrayValue!
    }

    /// interface as 1 by 2 matrix of floats
    public var interfaceShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(interface)
    }

    /// implementation as 1 by 2 matrix of floats
    public var implementation: MLMultiArray {
        provider.featureValue(for: "implementation")!.multiArrayValue!
    }

    /// implementation as 1 by 2 matrix of floats
    public var implementationShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(implementation)
    }

    /// end as 1 by 2 matrix of floats
    public var end: MLMultiArray {
        provider.featureValue(for: "end")!.multiArrayValue!
    }

    /// end as 1 by 2 matrix of floats
    public var endShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(end)
    }

    /// selector as 1 by 2 matrix of floats
    public var selector: MLMultiArray {
        provider.featureValue(for: "selector")!.multiArrayValue!
    }

    /// selector as 1 by 2 matrix of floats
    public var selectorShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(selector)
    }

    /// encode as 1 by 2 matrix of floats
    public var encode: MLMultiArray {
        provider.featureValue(for: "encode")!.multiArrayValue!
    }

    /// encode as 1 by 2 matrix of floats
    public var encodeShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(encode)
    }

    /// autoreleasepool as 1 by 2 matrix of floats
    public var autoreleasepool: MLMultiArray {
        provider.featureValue(for: "autoreleasepool")!.multiArrayValue!
    }

    /// autoreleasepool as 1 by 2 matrix of floats
    public var autoreleasepoolShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(autoreleasepool)
    }

    /// finally as 1 by 2 matrix of floats
    public var finally: MLMultiArray {
        provider.featureValue(for: "finally")!.multiArrayValue!
    }

    /// finally as 1 by 2 matrix of floats
    public var finallyShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(finally)
    }

    /// Int as 1 by 2 matrix of floats
    public var Int: MLMultiArray {
        provider.featureValue(for: "Int")!.multiArrayValue!
    }

    /// Int as 1 by 2 matrix of floats
    public var IntShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Int)
    }

    /// String as 1 by 2 matrix of floats
    public var String: MLMultiArray {
        provider.featureValue(for: "String")!.multiArrayValue!
    }

    /// String as 1 by 2 matrix of floats
    public var StringShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(String)
    }

    /// Float as 1 by 2 matrix of floats
    public var Float: MLMultiArray {
        provider.featureValue(for: "Float")!.multiArrayValue!
    }

    /// Float as 1 by 2 matrix of floats
    public var FloatShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Float)
    }

    /// Double as 1 by 2 matrix of floats
    public var Double: MLMultiArray {
        provider.featureValue(for: "Double")!.multiArrayValue!
    }

    /// Double as 1 by 2 matrix of floats
    public var DoubleShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Double)
    }

    /// Array as 1 by 2 matrix of floats
    public var Array: MLMultiArray {
        provider.featureValue(for: "Array")!.multiArrayValue!
    }

    /// Array as 1 by 2 matrix of floats
    public var ArrayShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Array)
    }

    /// Dictionary as 1 by 2 matrix of floats
    public var Dictionary: MLMultiArray {
        provider.featureValue(for: "Dictionary")!.multiArrayValue!
    }

    /// Dictionary as 1 by 2 matrix of floats
    public var DictionaryShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Dictionary)
    }

    /// Set as 1 by 2 matrix of floats
    public var Set: MLMultiArray {
        provider.featureValue(for: "Set")!.multiArrayValue!
    }

    /// Set as 1 by 2 matrix of floats
    public var SetShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Set)
    }

    /// Optional as 1 by 2 matrix of floats
    public var Optional: MLMultiArray {
        provider.featureValue(for: "Optional")!.multiArrayValue!
    }

    /// Optional as 1 by 2 matrix of floats
    public var OptionalShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Optional)
    }

    /// Error as 1 by 2 matrix of floats
    public var Error: MLMultiArray {
        provider.featureValue(for: "Error")!.multiArrayValue!
    }

    /// Error as 1 by 2 matrix of floats
    public var ErrorShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Error)
    }

    /// URL as 1 by 2 matrix of floats
    public var URL: MLMultiArray {
        provider.featureValue(for: "URL")!.multiArrayValue!
    }

    /// URL as 1 by 2 matrix of floats
    public var URLShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(URL)
    }

    /// MLModel as 1 by 2 matrix of floats
    public var MLModel: MLMultiArray {
        provider.featureValue(for: "MLModel")!.multiArrayValue!
    }

    /// MLModel as 1 by 2 matrix of floats
    public var MLModelShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(MLModel)
    }

    /// MLMultiArray as 1 by 2 matrix of floats
    public var MLMultiArray: MLMultiArray {
        provider.featureValue(for: "MLMultiArray")!.multiArrayValue!
    }

    /// MLMultiArray as 1 by 2 matrix of floats
    public var MLMultiArrayShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(MLMultiArray)
    }

    /// CVPixelBuffer as 1 by 2 matrix of floats
    public var CVPixelBuffer: MLMultiArray {
        provider.featureValue(for: "CVPixelBuffer")!.multiArrayValue!
    }

    /// CVPixelBuffer as 1 by 2 matrix of floats
    public var CVPixelBufferShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(CVPixelBuffer)
    }

    /// Bundle as 1 by 2 matrix of floats
    public var Bundle: MLMultiArray {
        provider.featureValue(for: "Bundle")!.multiArrayValue!
    }

    /// Bundle as 1 by 2 matrix of floats
    public var BundleShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Bundle)
    }

    /// result as 1 by 2 matrix of floats
    public var result: MLMultiArray {
        provider.featureValue(for: "result")!.multiArrayValue!
    }

    /// result as 1 by 2 matrix of floats
    public var resultShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(result)
    }

    /// results as 1 by 2 matrix of floats
    public var results: MLMultiArray {
        provider.featureValue(for: "results")!.multiArrayValue!
    }

    /// results as 1 by 2 matrix of floats
    public var resultsShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(results)
    }

    /// options as 1 by 2 matrix of floats
    public var options: MLMultiArray {
        provider.featureValue(for: "options")!.multiArrayValue!
    }

    /// options as 1 by 2 matrix of floats
    public var optionsShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(options)
    }

    /// configuration as 1 by 2 matrix of floats
    public var configuration: MLMultiArray {
        provider.featureValue(for: "configuration")!.multiArrayValue!
    }

    /// configuration as 1 by 2 matrix of floats
    public var configurationShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(configuration)
    }

    /// handler as 1 by 2 matrix of floats
    public var handler: MLMultiArray {
        provider.featureValue(for: "handler")!.multiArrayValue!
    }

    /// handler as 1 by 2 matrix of floats
    public var handlerShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(handler)
    }

    /// error as 1 by 2 matrix of floats
    public var error_: MLMultiArray {
        provider.featureValue(for: "error")!.multiArrayValue!
    }

    /// error as 1 by 2 matrix of floats
    public var error_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(error_)
    }

    /// batchIn as 1 by 2 matrix of floats
    public var batchIn: MLMultiArray {
        provider.featureValue(for: "batchIn")!.multiArrayValue!
    }

    /// batchIn as 1 by 2 matrix of floats
    public var batchInShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(batchIn)
    }

    /// outFeatures as 1 by 2 matrix of floats
    public var outFeatures: MLMultiArray {
        provider.featureValue(for: "outFeatures")!.multiArrayValue!
    }

    /// outFeatures as 1 by 2 matrix of floats
    public var outFeaturesShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(outFeatures)
    }

    /// localError as 1 by 2 matrix of floats
    public var localError: MLMultiArray {
        provider.featureValue(for: "localError")!.multiArrayValue!
    }

    /// localError as 1 by 2 matrix of floats
    public var localErrorShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(localError)
    }

    /// retVal as 1 by 2 matrix of floats
    public var retVal: MLMultiArray {
        provider.featureValue(for: "retVal")!.multiArrayValue!
    }

    /// retVal as 1 by 2 matrix of floats
    public var retValShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(retVal)
    }

    /// label as 1 by 2 matrix of floats
    public var label: MLMultiArray {
        provider.featureValue(for: "label")!.multiArrayValue!
    }

    /// label as 1 by 2 matrix of floats
    public var labelShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(label)
    }

    /// prediction as 1 by 2 matrix of floats
    public var prediction: MLMultiArray {
        provider.featureValue(for: "prediction")!.multiArrayValue!
    }

    /// prediction as 1 by 2 matrix of floats
    public var predictionShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(prediction)
    }

    /// predictions as 1 by 2 matrix of floats
    public var predictions: MLMultiArray {
        provider.featureValue(for: "predictions")!.multiArrayValue!
    }

    /// predictions as 1 by 2 matrix of floats
    public var predictionsShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(predictions)
    }

    /// urlOfModelInThisBundle as 1 by 2 matrix of floats
    public var urlOfModelInThisBundle: MLMultiArray {
        provider.featureValue(for: "urlOfModelInThisBundle")!.multiArrayValue!
    }

    /// urlOfModelInThisBundle as 1 by 2 matrix of floats
    public var urlOfModelInThisBundleShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(urlOfModelInThisBundle)
    }

    /// load as 1 by 2 matrix of floats
    public var load: MLMultiArray {
        provider.featureValue(for: "load")!.multiArrayValue!
    }

    /// load as 1 by 2 matrix of floats
    public var loadShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(load)
    }

    /// bundle as 1 by 2 matrix of floats
    public var bundle: MLMultiArray {
        provider.featureValue(for: "bundle")!.multiArrayValue!
    }

    /// bundle as 1 by 2 matrix of floats
    public var bundleShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(bundle)
    }

    /// precedencegroup as 1 by 2 matrix of floats
    public var precedencegroup: MLMultiArray {
        provider.featureValue(for: "precedencegroup")!.multiArrayValue!
    }

    /// precedencegroup as 1 by 2 matrix of floats
    public var precedencegroupShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(precedencegroup)
    }

    /// _Bool as 1 by 2 matrix of floats
    public var _Bool_: MLMultiArray {
        provider.featureValue(for: "_Bool")!.multiArrayValue!
    }

    /// _Bool as 1 by 2 matrix of floats
    public var _Bool_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(_Bool_)
    }

    /// static_assert as 1 by 2 matrix of floats
    public var static_assert: MLMultiArray {
        provider.featureValue(for: "static_assert")!.multiArrayValue!
    }

    /// static_assert as 1 by 2 matrix of floats
    public var static_assertShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(static_assert)
    }

    /// alignas as 1 by 2 matrix of floats
    public var alignas: MLMultiArray {
        provider.featureValue(for: "alignas")!.multiArrayValue!
    }

    /// alignas as 1 by 2 matrix of floats
    public var alignasShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(alignas)
    }

    /// alignof as 1 by 2 matrix of floats
    public var alignof: MLMultiArray {
        provider.featureValue(for: "alignof")!.multiArrayValue!
    }

    /// alignof as 1 by 2 matrix of floats
    public var alignofShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(alignof)
    }

    /// decltype as 1 by 2 matrix of floats
    public var decltype: MLMultiArray {
        provider.featureValue(for: "decltype")!.multiArrayValue!
    }

    /// decltype as 1 by 2 matrix of floats
    public var decltypeShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(decltype)
    }

    /// noexcept as 1 by 2 matrix of floats
    public var noexcept: MLMultiArray {
        provider.featureValue(for: "noexcept")!.multiArrayValue!
    }

    /// noexcept as 1 by 2 matrix of floats
    public var noexceptShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(noexcept)
    }

    /// nullptr as 1 by 2 matrix of floats
    public var nullptr: MLMultiArray {
        provider.featureValue(for: "nullptr")!.multiArrayValue!
    }

    /// nullptr as 1 by 2 matrix of floats
    public var nullptrShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(nullptr)
    }

    /// thread_local as 1 by 2 matrix of floats
    public var thread_local: MLMultiArray {
        provider.featureValue(for: "thread_local")!.multiArrayValue!
    }

    /// thread_local as 1 by 2 matrix of floats
    public var thread_localShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(thread_local)
    }

    /// constexpr as 1 by 2 matrix of floats
    public var constexpr: MLMultiArray {
        provider.featureValue(for: "constexpr")!.multiArrayValue!
    }

    /// constexpr as 1 by 2 matrix of floats
    public var constexprShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(constexpr)
    }

    /// char16_t as 1 by 2 matrix of floats
    public var char16_t: MLMultiArray {
        provider.featureValue(for: "char16_t")!.multiArrayValue!
    }

    /// char16_t as 1 by 2 matrix of floats
    public var char16_tShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(char16_t)
    }

    /// char32_t as 1 by 2 matrix of floats
    public var char32_t: MLMultiArray {
        provider.featureValue(for: "char32_t")!.multiArrayValue!
    }

    /// char32_t as 1 by 2 matrix of floats
    public var char32_tShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(char32_t)
    }

    /// wchar_t as 1 by 2 matrix of floats
    public var wchar_t: MLMultiArray {
        provider.featureValue(for: "wchar_t")!.multiArrayValue!
    }

    /// wchar_t as 1 by 2 matrix of floats
    public var wchar_tShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(wchar_t)
    }

    /// dynamicType as 1 by 2 matrix of floats
    public var dynamicType_: MLMultiArray {
        provider.featureValue(for: "dynamicType")!.multiArrayValue!
    }

    /// dynamicType as 1 by 2 matrix of floats
    public var dynamicType_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(dynamicType_)
    }

    /// Category as 1 by 2 matrix of floats
    public var Category: MLMultiArray {
        provider.featureValue(for: "Category")!.multiArrayValue!
    }

    /// Category as 1 by 2 matrix of floats
    public var CategoryShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Category)
    }

    /// Method as 1 by 2 matrix of floats
    public var Method: MLMultiArray {
        provider.featureValue(for: "Method")!.multiArrayValue!
    }

    /// Method as 1 by 2 matrix of floats
    public var MethodShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Method)
    }

    /// Ivar as 1 by 2 matrix of floats
    public var Ivar: MLMultiArray {
        provider.featureValue(for: "Ivar")!.multiArrayValue!
    }

    /// Ivar as 1 by 2 matrix of floats
    public var IvarShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Ivar)
    }

    /// unknown as 1 by 2 matrix of floats
    public var unknown: MLMultiArray {
        provider.featureValue(for: "unknown")!.multiArrayValue!
    }

    /// unknown as 1 by 2 matrix of floats
    public var unknownShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(unknown)
    }

    /// and as 1 by 2 matrix of floats
    public var and: MLMultiArray {
        provider.featureValue(for: "and")!.multiArrayValue!
    }

    /// and as 1 by 2 matrix of floats
    public var andShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(and)
    }

    /// or as 1 by 2 matrix of floats
    public var or: MLMultiArray {
        provider.featureValue(for: "or")!.multiArrayValue!
    }

    /// or as 1 by 2 matrix of floats
    public var orShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(or)
    }

    /// not as 1 by 2 matrix of floats
    public var not: MLMultiArray {
        provider.featureValue(for: "not")!.multiArrayValue!
    }

    /// not as 1 by 2 matrix of floats
    public var notShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(not)
    }

    /// xor as 1 by 2 matrix of floats
    public var xor: MLMultiArray {
        provider.featureValue(for: "xor")!.multiArrayValue!
    }

    /// xor as 1 by 2 matrix of floats
    public var xorShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(xor)
    }

    /// bitand as 1 by 2 matrix of floats
    public var bitand: MLMultiArray {
        provider.featureValue(for: "bitand")!.multiArrayValue!
    }

    /// bitand as 1 by 2 matrix of floats
    public var bitandShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(bitand)
    }

    /// bitor as 1 by 2 matrix of floats
    public var bitor: MLMultiArray {
        provider.featureValue(for: "bitor")!.multiArrayValue!
    }

    /// bitor as 1 by 2 matrix of floats
    public var bitorShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(bitor)
    }

    /// compl as 1 by 2 matrix of floats
    public var compl: MLMultiArray {
        provider.featureValue(for: "compl")!.multiArrayValue!
    }

    /// compl as 1 by 2 matrix of floats
    public var complShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(compl)
    }

    /// not_eq as 1 by 2 matrix of floats
    public var not_eq: MLMultiArray {
        provider.featureValue(for: "not_eq")!.multiArrayValue!
    }

    /// not_eq as 1 by 2 matrix of floats
    public var not_eqShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(not_eq)
    }

    /// or_eq as 1 by 2 matrix of floats
    public var or_eq: MLMultiArray {
        provider.featureValue(for: "or_eq")!.multiArrayValue!
    }

    /// or_eq as 1 by 2 matrix of floats
    public var or_eqShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(or_eq)
    }

    /// and_eq as 1 by 2 matrix of floats
    public var and_eq: MLMultiArray {
        provider.featureValue(for: "and_eq")!.multiArrayValue!
    }

    /// and_eq as 1 by 2 matrix of floats
    public var and_eqShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(and_eq)
    }

    /// xor_eq as 1 by 2 matrix of floats
    public var xor_eq: MLMultiArray {
        provider.featureValue(for: "xor_eq")!.multiArrayValue!
    }

    /// xor_eq as 1 by 2 matrix of floats
    public var xor_eqShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(xor_eq)
    }

    /// typeid as 1 by 2 matrix of floats
    public var typeid: MLMultiArray {
        provider.featureValue(for: "typeid")!.multiArrayValue!
    }

    /// typeid as 1 by 2 matrix of floats
    public var typeidShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(typeid)
    }

    /// typename as 1 by 2 matrix of floats
    public var typename: MLMultiArray {
        provider.featureValue(for: "typename")!.multiArrayValue!
    }

    /// typename as 1 by 2 matrix of floats
    public var typenameShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(typename)
    }

    /// using as 1 by 2 matrix of floats
    public var using: MLMultiArray {
        provider.featureValue(for: "using")!.multiArrayValue!
    }

    /// using as 1 by 2 matrix of floats
    public var usingShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(using)
    }

    /// public_ as 1 by 2 matrix of floats
    public var public_: MLMultiArray {
        provider.featureValue(for: "public_")!.multiArrayValue!
    }

    /// public_ as 1 by 2 matrix of floats
    public var public_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(public_)
    }

    /// throws_ as 1 by 2 matrix of floats
    public var throws_: MLMultiArray {
        provider.featureValue(for: "throws_")!.multiArrayValue!
    }

    /// throws_ as 1 by 2 matrix of floats
    public var throws_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(throws_)
    }

    /// mutating_ as 1 by 2 matrix of floats
    public var mutating_: MLMultiArray {
        provider.featureValue(for: "mutating_")!.multiArrayValue!
    }

    /// mutating_ as 1 by 2 matrix of floats
    public var mutating_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(mutating_)
    }

    /// Int32 as 1 by 2 matrix of floats
    public var Int32: MLMultiArray {
        provider.featureValue(for: "Int32")!.multiArrayValue!
    }

    /// Int32 as 1 by 2 matrix of floats
    public var Int32ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Int32)
    }

    /// Int64 as 1 by 2 matrix of floats
    public var Int64: MLMultiArray {
        provider.featureValue(for: "Int64")!.multiArrayValue!
    }

    /// Int64 as 1 by 2 matrix of floats
    public var Int64ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(Int64)
    }

    /// UInt as 1 by 2 matrix of floats
    public var UInt: MLMultiArray {
        provider.featureValue(for: "UInt")!.multiArrayValue!
    }

    /// UInt as 1 by 2 matrix of floats
    public var UIntShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(UInt)
    }

    /// NSObject as 1 by 2 matrix of floats
    public var NSObject: MLMultiArray {
        provider.featureValue(for: "NSObject")!.multiArrayValue!
    }

    /// NSObject as 1 by 2 matrix of floats
    public var NSObjectShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(NSObject)
    }

    /// NSString as 1 by 2 matrix of floats
    public var NSString: MLMultiArray {
        provider.featureValue(for: "NSString")!.multiArrayValue!
    }

    /// NSString as 1 by 2 matrix of floats
    public var NSStringShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(NSString)
    }

    /// NSNumber as 1 by 2 matrix of floats
    public var NSNumber: MLMultiArray {
        provider.featureValue(for: "NSNumber")!.multiArrayValue!
    }

    /// NSNumber as 1 by 2 matrix of floats
    public var NSNumberShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(NSNumber)
    }

    /// NSError as 1 by 2 matrix of floats
    public var NSError: MLMultiArray {
        provider.featureValue(for: "NSError")!.multiArrayValue!
    }

    /// NSError as 1 by 2 matrix of floats
    public var NSErrorShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(NSError)
    }

    /// instancetype as 1 by 2 matrix of floats
    public var instancetype: MLMultiArray {
        provider.featureValue(for: "instancetype")!.multiArrayValue!
    }

    /// instancetype as 1 by 2 matrix of floats
    public var instancetypeShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(instancetype)
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

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(catch: MLMultiArray, package: MLMultiArray, macro: MLMultiArray, borrowing: MLMultiArray, consuming: MLMultiArray, nonisolated: MLMultiArray, isolated: MLMultiArray, actor: MLMultiArray, each: MLMultiArray, asm: MLMultiArray, typeof: MLMultiArray, nullable: MLMultiArray, nonnull: MLMultiArray, property: MLMultiArray, synthesize: MLMultiArray, interface: MLMultiArray, implementation: MLMultiArray, end: MLMultiArray, selector: MLMultiArray, encode: MLMultiArray, autoreleasepool: MLMultiArray, finally: MLMultiArray, Int: MLMultiArray, String: MLMultiArray, Float: MLMultiArray, Double: MLMultiArray, Array: MLMultiArray, Dictionary: MLMultiArray, Set: MLMultiArray, Optional: MLMultiArray, Error: MLMultiArray, URL: MLMultiArray, MLModel: MLMultiArray, MLMultiArray: MLMultiArray, CVPixelBuffer: MLMultiArray, Bundle: MLMultiArray, result: MLMultiArray, results: MLMultiArray, options: MLMultiArray, configuration: MLMultiArray, handler: MLMultiArray, error_: MLMultiArray, batchIn: MLMultiArray, outFeatures: MLMultiArray, localError: MLMultiArray, retVal: MLMultiArray, label: MLMultiArray, prediction: MLMultiArray, predictions: MLMultiArray, urlOfModelInThisBundle: MLMultiArray, load: MLMultiArray, bundle: MLMultiArray, precedencegroup: MLMultiArray, _Bool_: MLMultiArray, static_assert: MLMultiArray, alignas: MLMultiArray, alignof: MLMultiArray, decltype: MLMultiArray, noexcept: MLMultiArray, nullptr: MLMultiArray, thread_local: MLMultiArray, constexpr: MLMultiArray, char16_t: MLMultiArray, char32_t: MLMultiArray, wchar_t: MLMultiArray, dynamicType_: MLMultiArray, Category: MLMultiArray, Method: MLMultiArray, Ivar: MLMultiArray, unknown: MLMultiArray, and: MLMultiArray, or: MLMultiArray, not: MLMultiArray, xor: MLMultiArray, bitand: MLMultiArray, bitor: MLMultiArray, compl: MLMultiArray, not_eq: MLMultiArray, or_eq: MLMultiArray, and_eq: MLMultiArray, xor_eq: MLMultiArray, typeid: MLMultiArray, typename: MLMultiArray, using: MLMultiArray, public_: MLMultiArray, throws_: MLMultiArray, mutating_: MLMultiArray, Int32: MLMultiArray, Int64: MLMultiArray, UInt: MLMultiArray, NSObject: MLMultiArray, NSString: MLMultiArray, NSNumber: MLMultiArray, NSError: MLMultiArray, instancetype: MLMultiArray, strong: MLMultiArray, copy: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["catch" : MLFeatureValue(multiArray: catch), "package" : MLFeatureValue(multiArray: package), "macro" : MLFeatureValue(multiArray: macro), "borrowing" : MLFeatureValue(multiArray: borrowing), "consuming" : MLFeatureValue(multiArray: consuming), "nonisolated" : MLFeatureValue(multiArray: nonisolated), "isolated" : MLFeatureValue(multiArray: isolated), "actor" : MLFeatureValue(multiArray: actor), "each" : MLFeatureValue(multiArray: each), "asm" : MLFeatureValue(multiArray: asm), "typeof" : MLFeatureValue(multiArray: typeof), "nullable" : MLFeatureValue(multiArray: nullable), "nonnull" : MLFeatureValue(multiArray: nonnull), "property" : MLFeatureValue(multiArray: property), "synthesize" : MLFeatureValue(multiArray: synthesize), "interface" : MLFeatureValue(multiArray: interface), "implementation" : MLFeatureValue(multiArray: implementation), "end" : MLFeatureValue(multiArray: end), "selector" : MLFeatureValue(multiArray: selector), "encode" : MLFeatureValue(multiArray: encode), "autoreleasepool" : MLFeatureValue(multiArray: autoreleasepool), "finally" : MLFeatureValue(multiArray: finally), "Int" : MLFeatureValue(multiArray: Int), "String" : MLFeatureValue(multiArray: String), "Float" : MLFeatureValue(multiArray: Float), "Double" : MLFeatureValue(multiArray: Double), "Array" : MLFeatureValue(multiArray: Array), "Dictionary" : MLFeatureValue(multiArray: Dictionary), "Set" : MLFeatureValue(multiArray: Set), "Optional" : MLFeatureValue(multiArray: Optional), "Error" : MLFeatureValue(multiArray: Error), "URL" : MLFeatureValue(multiArray: URL), "MLModel" : MLFeatureValue(multiArray: MLModel), "MLMultiArray" : MLFeatureValue(multiArray: MLMultiArray), "CVPixelBuffer" : MLFeatureValue(multiArray: CVPixelBuffer), "Bundle" : MLFeatureValue(multiArray: Bundle), "result" : MLFeatureValue(multiArray: result), "results" : MLFeatureValue(multiArray: results), "options" : MLFeatureValue(multiArray: options), "configuration" : MLFeatureValue(multiArray: configuration), "handler" : MLFeatureValue(multiArray: handler), "error" : MLFeatureValue(multiArray: error_), "batchIn" : MLFeatureValue(multiArray: batchIn), "outFeatures" : MLFeatureValue(multiArray: outFeatures), "localError" : MLFeatureValue(multiArray: localError), "retVal" : MLFeatureValue(multiArray: retVal), "label" : MLFeatureValue(multiArray: label), "prediction" : MLFeatureValue(multiArray: prediction), "predictions" : MLFeatureValue(multiArray: predictions), "urlOfModelInThisBundle" : MLFeatureValue(multiArray: urlOfModelInThisBundle), "load" : MLFeatureValue(multiArray: load), "bundle" : MLFeatureValue(multiArray: bundle), "precedencegroup" : MLFeatureValue(multiArray: precedencegroup), "_Bool" : MLFeatureValue(multiArray: _Bool_), "static_assert" : MLFeatureValue(multiArray: static_assert), "alignas" : MLFeatureValue(multiArray: alignas), "alignof" : MLFeatureValue(multiArray: alignof), "decltype" : MLFeatureValue(multiArray: decltype), "noexcept" : MLFeatureValue(multiArray: noexcept), "nullptr" : MLFeatureValue(multiArray: nullptr), "thread_local" : MLFeatureValue(multiArray: thread_local), "constexpr" : MLFeatureValue(multiArray: constexpr), "char16_t" : MLFeatureValue(multiArray: char16_t), "char32_t" : MLFeatureValue(multiArray: char32_t), "wchar_t" : MLFeatureValue(multiArray: wchar_t), "dynamicType" : MLFeatureValue(multiArray: dynamicType_), "Category" : MLFeatureValue(multiArray: Category), "Method" : MLFeatureValue(multiArray: Method), "Ivar" : MLFeatureValue(multiArray: Ivar), "unknown" : MLFeatureValue(multiArray: unknown), "and" : MLFeatureValue(multiArray: and), "or" : MLFeatureValue(multiArray: or), "not" : MLFeatureValue(multiArray: not), "xor" : MLFeatureValue(multiArray: xor), "bitand" : MLFeatureValue(multiArray: bitand), "bitor" : MLFeatureValue(multiArray: bitor), "compl" : MLFeatureValue(multiArray: compl), "not_eq" : MLFeatureValue(multiArray: not_eq), "or_eq" : MLFeatureValue(multiArray: or_eq), "and_eq" : MLFeatureValue(multiArray: and_eq), "xor_eq" : MLFeatureValue(multiArray: xor_eq), "typeid" : MLFeatureValue(multiArray: typeid), "typename" : MLFeatureValue(multiArray: typename), "using" : MLFeatureValue(multiArray: using), "public_" : MLFeatureValue(multiArray: public_), "throws_" : MLFeatureValue(multiArray: throws_), "mutating_" : MLFeatureValue(multiArray: mutating_), "Int32" : MLFeatureValue(multiArray: Int32), "Int64" : MLFeatureValue(multiArray: Int64), "UInt" : MLFeatureValue(multiArray: UInt), "NSObject" : MLFeatureValue(multiArray: NSObject), "NSString" : MLFeatureValue(multiArray: NSString), "NSNumber" : MLFeatureValue(multiArray: NSNumber), "NSError" : MLFeatureValue(multiArray: NSError), "instancetype" : MLFeatureValue(multiArray: instancetype), "strong" : MLFeatureValue(multiArray: strong), "copy" : MLFeatureValue(multiArray: copy)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class Names2 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Names2", withExtension:"mlmodelc")!
    }

    /**
        Construct Names2 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Names2.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Names2.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct Names2 instance with explicit path to mlmodelc file
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
        Construct Names2 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Names2, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Names2 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Names2 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Names2 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Names2, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Names2(model: model)))
            }
        }
    }

    /**
        Construct Names2 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Names2 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Names2(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Names2Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Names2Output
    */
    public func prediction(input: Names2Input) throws -> Names2Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Names2Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Names2Output
    */
    public func prediction(input: Names2Input, options: MLPredictionOptions) throws -> Names2Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return Names2Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Names2Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Names2Output
    */
    public func prediction(input: Names2Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> Names2Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return Names2Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 by 2 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Names2Output
    */
    public func prediction(x: MLMultiArray) throws -> Names2Output {
        let input_ = Names2Input(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 by 2 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Names2Output
    */

    public func prediction(x: MLShapedArray<Float>) throws -> Names2Output {
        let input_ = Names2Input(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [Names2Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [Names2Output]
    */
    public func predictions(inputs: [Names2Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [Names2Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [Names2Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  Names2Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
