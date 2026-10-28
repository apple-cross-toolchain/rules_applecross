//
// Multi.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class MultiInput : MLFeatureProvider {

    /// the a as 3 element vector of floats
    public var a: MLMultiArray

    /// the b as 2 × 3 × 4 3-dimensional array of 32-bit integers
    public var b: MLMultiArray

    public var featureNames: Set<String> { ["a", "b"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "a" {
            return MLFeatureValue(multiArray: a)
        }
        if featureName == "b" {
            return MLFeatureValue(multiArray: b)
        }
        return nil
    }

    public init(a: MLMultiArray, b: MLMultiArray) {
        self.a = a
        self.b = b
    }

    public convenience init(a: MLShapedArray<Float>, b: MLShapedArray<Int32>) {
        self.init(a: MLMultiArray(a), b: MLMultiArray(b))
    }

}


/// Model Prediction Output Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class MultiOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// the out as 3 element vector of floats
    public var out_: MLMultiArray {
        provider.featureValue(for: "out")!.multiArrayValue!
    }

    /// the out as 3 element vector of floats
    public var out_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(out_)
    }

    /// the var as 2 × 3 × 4 3-dimensional array of floats
    public var var_: MLMultiArray {
        provider.featureValue(for: "var")!.multiArrayValue!
    }

    /// the var as 2 × 3 × 4 3-dimensional array of floats
    public var var_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(var_)
    }

    /// the total as multidimensional array of floats
    public var total: MLMultiArray {
        provider.featureValue(for: "total")!.multiArrayValue!
    }

    /// the total as multidimensional array of floats
    public var totalShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(total)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(out_: MLMultiArray, var_: MLMultiArray, total: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["out" : MLFeatureValue(multiArray: out_), "var" : MLFeatureValue(multiArray: var_), "total" : MLFeatureValue(multiArray: total)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class Multi {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Multi", withExtension:"mlmodelc")!
    }

    /**
        Construct Multi instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Multi.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Multi.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct Multi instance with explicit path to mlmodelc file
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
        Construct Multi instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Multi, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Multi instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Multi {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Multi instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Multi, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Multi(model: model)))
            }
        }
    }

    /**
        Construct Multi instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Multi {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Multi(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as MultiInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as MultiOutput
    */
    public func prediction(input: MultiInput) throws -> MultiOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as MultiInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as MultiOutput
    */
    public func prediction(input: MultiInput, options: MLPredictionOptions) throws -> MultiOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return MultiOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as MultiInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as MultiOutput
    */
    public func prediction(input: MultiInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> MultiOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return MultiOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - a: the a as 3 element vector of floats
            - b: the b as 2 × 3 × 4 3-dimensional array of 32-bit integers

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as MultiOutput
    */
    public func prediction(a: MLMultiArray, b: MLMultiArray) throws -> MultiOutput {
        let input_ = MultiInput(a: a, b: b)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - a: the a as 3 element vector of floats
            - b: the b as 2 × 3 × 4 3-dimensional array of 32-bit integers

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as MultiOutput
    */

    public func prediction(a: MLShapedArray<Float>, b: MLShapedArray<Int32>) throws -> MultiOutput {
        let input_ = MultiInput(a: a, b: b)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [MultiInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [MultiOutput]
    */
    public func predictions(inputs: [MultiInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [MultiOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [MultiOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  MultiOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
