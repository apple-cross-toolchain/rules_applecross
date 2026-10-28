//
// F15_negnan.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
public class F15_negnanInput : MLFeatureProvider {

    /// x as 4 element vector of floats
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
@available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
public class F15_negnanOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// add_0 as 4 element vector of floats
    public var add_0: MLMultiArray {
        provider.featureValue(for: "add_0")!.multiArrayValue!
    }

    /// add_0 as 4 element vector of floats
    public var add_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(add_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(add_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["add_0" : MLFeatureValue(multiArray: add_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
public class F15_negnan {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "F15_negnan", withExtension:"mlmodelc")!
    }

    /**
        Construct F15_negnan instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of F15_negnan.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `F15_negnan.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct F15_negnan instance with explicit path to mlmodelc file
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
        Construct F15_negnan instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<F15_negnan, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct F15_negnan instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> F15_negnan {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct F15_negnan instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<F15_negnan, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(F15_negnan(model: model)))
            }
        }
    }

    /**
        Construct F15_negnan instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> F15_negnan {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return F15_negnan(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as F15_negnanInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F15_negnanOutput
    */
    public func prediction(input: F15_negnanInput) throws -> F15_negnanOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as F15_negnanInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F15_negnanOutput
    */
    public func prediction(input: F15_negnanInput, options: MLPredictionOptions) throws -> F15_negnanOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return F15_negnanOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as F15_negnanInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F15_negnanOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: F15_negnanInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> F15_negnanOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return F15_negnanOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 4 element vector of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F15_negnanOutput
    */
    public func prediction(x: MLMultiArray) throws -> F15_negnanOutput {
        let input_ = F15_negnanInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 4 element vector of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F15_negnanOutput
    */

    public func prediction(x: MLShapedArray<Float>) throws -> F15_negnanOutput {
        let input_ = F15_negnanInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [F15_negnanInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [F15_negnanOutput]
    */
    public func predictions(inputs: [F15_negnanInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [F15_negnanOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [F15_negnanOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  F15_negnanOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
