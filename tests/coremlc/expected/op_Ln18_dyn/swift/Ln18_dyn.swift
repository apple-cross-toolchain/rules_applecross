//
// Ln18_dyn.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class Ln18_dynInput : MLFeatureProvider {

    /// x as 2 by 3 matrix of floats
    public var x: MLMultiArray

    /// y as 1 element vector of 32-bit integers
    public var y: MLMultiArray

    public var featureNames: Set<String> { ["x", "y"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "x" {
            return MLFeatureValue(multiArray: x)
        }
        if featureName == "y" {
            return MLFeatureValue(multiArray: y)
        }
        return nil
    }

    public init(x: MLMultiArray, y: MLMultiArray) {
        self.x = x
        self.y = y
    }

    public convenience init(x: MLShapedArray<Float>, y: MLShapedArray<Int32>) {
        self.init(x: MLMultiArray(x), y: MLMultiArray(y))
    }

}


/// Model Prediction Output Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class Ln18_dynOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// list_read_0 as 2 by 3 matrix of floats
    public var list_read_0: MLMultiArray {
        provider.featureValue(for: "list_read_0")!.multiArrayValue!
    }

    /// list_read_0 as 2 by 3 matrix of floats
    public var list_read_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(list_read_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(list_read_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["list_read_0" : MLFeatureValue(multiArray: list_read_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class Ln18_dyn {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Ln18_dyn", withExtension:"mlmodelc")!
    }

    /**
        Construct Ln18_dyn instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Ln18_dyn.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Ln18_dyn.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct Ln18_dyn instance with explicit path to mlmodelc file
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
        Construct Ln18_dyn instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Ln18_dyn, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Ln18_dyn instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Ln18_dyn {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Ln18_dyn instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Ln18_dyn, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Ln18_dyn(model: model)))
            }
        }
    }

    /**
        Construct Ln18_dyn instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Ln18_dyn {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Ln18_dyn(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Ln18_dynInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Ln18_dynOutput
    */
    public func prediction(input: Ln18_dynInput) throws -> Ln18_dynOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Ln18_dynInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Ln18_dynOutput
    */
    public func prediction(input: Ln18_dynInput, options: MLPredictionOptions) throws -> Ln18_dynOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return Ln18_dynOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Ln18_dynInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Ln18_dynOutput
    */
    public func prediction(input: Ln18_dynInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> Ln18_dynOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return Ln18_dynOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 2 by 3 matrix of floats
            - y: 1 element vector of 32-bit integers

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Ln18_dynOutput
    */
    public func prediction(x: MLMultiArray, y: MLMultiArray) throws -> Ln18_dynOutput {
        let input_ = Ln18_dynInput(x: x, y: y)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 2 by 3 matrix of floats
            - y: 1 element vector of 32-bit integers

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Ln18_dynOutput
    */

    public func prediction(x: MLShapedArray<Float>, y: MLShapedArray<Int32>) throws -> Ln18_dynOutput {
        let input_ = Ln18_dynInput(x: x, y: y)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [Ln18_dynInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [Ln18_dynOutput]
    */
    public func predictions(inputs: [Ln18_dynInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [Ln18_dynOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [Ln18_dynOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  Ln18_dynOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
