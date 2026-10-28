//
// Enum18.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class Enum18Input : MLFeatureProvider {

    /// x as 1 by 3 matrix of floats
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
public class Enum18Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// relu_0 as multidimensional array of floats
    public var relu_0: MLMultiArray {
        provider.featureValue(for: "relu_0")!.multiArrayValue!
    }

    /// relu_0 as multidimensional array of floats
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
public class Enum18 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Enum18", withExtension:"mlmodelc")!
    }

    /**
        Construct Enum18 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Enum18.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Enum18.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct Enum18 instance with explicit path to mlmodelc file
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
        Construct Enum18 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Enum18, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Enum18 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Enum18 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Enum18 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Enum18, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Enum18(model: model)))
            }
        }
    }

    /**
        Construct Enum18 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Enum18 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Enum18(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Enum18Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Enum18Output
    */
    public func prediction(input: Enum18Input) throws -> Enum18Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Enum18Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Enum18Output
    */
    public func prediction(input: Enum18Input, options: MLPredictionOptions) throws -> Enum18Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return Enum18Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Enum18Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Enum18Output
    */
    public func prediction(input: Enum18Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> Enum18Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return Enum18Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 by 3 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Enum18Output
    */
    public func prediction(x: MLMultiArray) throws -> Enum18Output {
        let input_ = Enum18Input(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 1 by 3 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Enum18Output
    */

    public func prediction(x: MLShapedArray<Float>) throws -> Enum18Output {
        let input_ = Enum18Input(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [Enum18Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [Enum18Output]
    */
    public func predictions(inputs: [Enum18Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [Enum18Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [Enum18Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  Enum18Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
