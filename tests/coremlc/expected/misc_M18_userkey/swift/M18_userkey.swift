//
// M18_userkey.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class M18_userkeyInput : MLFeatureProvider {

    /// x as 2 element vector of floats
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
public class M18_userkeyOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// relu_0 as 2 element vector of floats
    public var relu_0: MLMultiArray {
        provider.featureValue(for: "relu_0")!.multiArrayValue!
    }

    /// relu_0 as 2 element vector of floats
    public var relu_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(relu_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(relu_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["relu_0" : MLFeatureValue(multiArray: relu_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class M18_userkey {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "M18_userkey", withExtension:"mlmodelc")!
    }

    /**
        Construct M18_userkey instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of M18_userkey.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `M18_userkey.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct M18_userkey instance with explicit path to mlmodelc file
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
        Construct M18_userkey instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<M18_userkey, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct M18_userkey instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> M18_userkey {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct M18_userkey instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<M18_userkey, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(M18_userkey(model: model)))
            }
        }
    }

    /**
        Construct M18_userkey instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> M18_userkey {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return M18_userkey(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as M18_userkeyInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as M18_userkeyOutput
    */
    public func prediction(input: M18_userkeyInput) throws -> M18_userkeyOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as M18_userkeyInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as M18_userkeyOutput
    */
    public func prediction(input: M18_userkeyInput, options: MLPredictionOptions) throws -> M18_userkeyOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return M18_userkeyOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as M18_userkeyInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as M18_userkeyOutput
    */
    public func prediction(input: M18_userkeyInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> M18_userkeyOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return M18_userkeyOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 2 element vector of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as M18_userkeyOutput
    */
    public func prediction(x: MLMultiArray) throws -> M18_userkeyOutput {
        let input_ = M18_userkeyInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 2 element vector of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as M18_userkeyOutput
    */

    public func prediction(x: MLShapedArray<Float>) throws -> M18_userkeyOutput {
        let input_ = M18_userkeyInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [M18_userkeyInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [M18_userkeyOutput]
    */
    public func predictions(inputs: [M18_userkeyInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [M18_userkeyOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [M18_userkeyOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  M18_userkeyOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
