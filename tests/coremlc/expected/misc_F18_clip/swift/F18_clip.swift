//
// F18_clip.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class F18_clipInput : MLFeatureProvider {

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
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class F18_clipOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// clip_0 as 4 element vector of floats
    public var clip_0: MLMultiArray {
        provider.featureValue(for: "clip_0")!.multiArrayValue!
    }

    /// clip_0 as 4 element vector of floats
    public var clip_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(clip_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(clip_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["clip_0" : MLFeatureValue(multiArray: clip_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class F18_clip {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "F18_clip", withExtension:"mlmodelc")!
    }

    /**
        Construct F18_clip instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of F18_clip.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `F18_clip.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct F18_clip instance with explicit path to mlmodelc file
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
        Construct F18_clip instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<F18_clip, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct F18_clip instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> F18_clip {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct F18_clip instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<F18_clip, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(F18_clip(model: model)))
            }
        }
    }

    /**
        Construct F18_clip instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> F18_clip {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return F18_clip(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as F18_clipInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F18_clipOutput
    */
    public func prediction(input: F18_clipInput) throws -> F18_clipOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as F18_clipInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F18_clipOutput
    */
    public func prediction(input: F18_clipInput, options: MLPredictionOptions) throws -> F18_clipOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return F18_clipOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as F18_clipInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F18_clipOutput
    */
    public func prediction(input: F18_clipInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> F18_clipOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return F18_clipOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 4 element vector of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F18_clipOutput
    */
    public func prediction(x: MLMultiArray) throws -> F18_clipOutput {
        let input_ = F18_clipInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 4 element vector of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as F18_clipOutput
    */

    public func prediction(x: MLShapedArray<Float>) throws -> F18_clipOutput {
        let input_ = F18_clipInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [F18_clipInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [F18_clipOutput]
    */
    public func predictions(inputs: [F18_clipInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [F18_clipOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [F18_clipOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  F18_clipOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
