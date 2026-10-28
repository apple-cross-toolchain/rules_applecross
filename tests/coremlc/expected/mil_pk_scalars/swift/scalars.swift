//
// scalars.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class scalarsInput : MLFeatureProvider {

    /// x as 2 by 3 matrix of floats
    public var x: MLMultiArray

    /// i as 2 element vector of 32-bit integers
    public var i: MLMultiArray

    public var featureNames: Set<String> { ["x", "i"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "x" {
            return MLFeatureValue(multiArray: x)
        }
        if featureName == "i" {
            return MLFeatureValue(multiArray: i)
        }
        return nil
    }

    public init(x: MLMultiArray, i: MLMultiArray) {
        self.x = x
        self.i = i
    }

    public convenience init(x: MLShapedArray<Float>, i: MLShapedArray<Int32>) {
        self.init(x: MLMultiArray(x), i: MLMultiArray(i))
    }

}


/// Model Prediction Output Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class scalarsOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// reshape_0 as 6 element vector of floats
    public var reshape_0: MLMultiArray {
        provider.featureValue(for: "reshape_0")!.multiArrayValue!
    }

    /// reshape_0 as 6 element vector of floats
    public var reshape_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(reshape_0)
    }

    /// cast_0 as 2 element vector of floats
    public var cast_0: MLMultiArray {
        provider.featureValue(for: "cast_0")!.multiArrayValue!
    }

    /// cast_0 as 2 element vector of floats
    public var cast_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(cast_0)
    }

    /// greater_0 as multidimensional array of floats
    public var greater_0: MLMultiArray {
        provider.featureValue(for: "greater_0")!.multiArrayValue!
    }

    /// greater_0 as multidimensional array of floats
    public var greater_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(greater_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(reshape_0: MLMultiArray, cast_0: MLMultiArray, greater_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["reshape_0" : MLFeatureValue(multiArray: reshape_0), "cast_0" : MLFeatureValue(multiArray: cast_0), "greater_0" : MLFeatureValue(multiArray: greater_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class scalars {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "scalars", withExtension:"mlmodelc")!
    }

    /**
        Construct scalars instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of scalars.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `scalars.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct scalars instance with explicit path to mlmodelc file
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
        Construct scalars instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<scalars, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct scalars instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> scalars {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct scalars instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<scalars, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(scalars(model: model)))
            }
        }
    }

    /**
        Construct scalars instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> scalars {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return scalars(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as scalarsInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as scalarsOutput
    */
    public func prediction(input: scalarsInput) throws -> scalarsOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as scalarsInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as scalarsOutput
    */
    public func prediction(input: scalarsInput, options: MLPredictionOptions) throws -> scalarsOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return scalarsOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as scalarsInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as scalarsOutput
    */
    public func prediction(input: scalarsInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> scalarsOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return scalarsOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 2 by 3 matrix of floats
            - i: 2 element vector of 32-bit integers

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as scalarsOutput
    */
    public func prediction(x: MLMultiArray, i: MLMultiArray) throws -> scalarsOutput {
        let input_ = scalarsInput(x: x, i: i)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 2 by 3 matrix of floats
            - i: 2 element vector of 32-bit integers

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as scalarsOutput
    */

    public func prediction(x: MLShapedArray<Float>, i: MLShapedArray<Int32>) throws -> scalarsOutput {
        let input_ = scalarsInput(x: x, i: i)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [scalarsInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [scalarsOutput]
    */
    public func predictions(inputs: [scalarsInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [scalarsOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [scalarsOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  scalarsOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
