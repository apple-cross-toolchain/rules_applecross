//
// h_iOS15_1.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
public class h_iOS15_1Input : MLFeatureProvider {

    /// a as 1 by 3 matrix of floats
    public var a: MLMultiArray

    public var featureNames: Set<String> { ["a"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "a" {
            return MLFeatureValue(multiArray: a)
        }
        return nil
    }

    public init(a: MLMultiArray) {
        self.a = a
    }

    public convenience init(a: MLShapedArray<Float>) {
        self.init(a: MLMultiArray(a))
    }

}


/// Model Prediction Output Type
@available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
public class h_iOS15_1Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// out as 1 by 3 matrix of floats
    public var out_: MLMultiArray {
        provider.featureValue(for: "out")!.multiArrayValue!
    }

    /// out as 1 by 3 matrix of floats
    public var out_ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(out_)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(out_: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["out" : MLFeatureValue(multiArray: out_)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
public class h_iOS15_1 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "h_iOS15_1", withExtension:"mlmodelc")!
    }

    /**
        Construct h_iOS15_1 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of h_iOS15_1.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `h_iOS15_1.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct h_iOS15_1 instance with explicit path to mlmodelc file
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
        Construct h_iOS15_1 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<h_iOS15_1, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct h_iOS15_1 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> h_iOS15_1 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct h_iOS15_1 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<h_iOS15_1, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(h_iOS15_1(model: model)))
            }
        }
    }

    /**
        Construct h_iOS15_1 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> h_iOS15_1 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return h_iOS15_1(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as h_iOS15_1Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as h_iOS15_1Output
    */
    public func prediction(input: h_iOS15_1Input) throws -> h_iOS15_1Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as h_iOS15_1Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as h_iOS15_1Output
    */
    public func prediction(input: h_iOS15_1Input, options: MLPredictionOptions) throws -> h_iOS15_1Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return h_iOS15_1Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as h_iOS15_1Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as h_iOS15_1Output
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: h_iOS15_1Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> h_iOS15_1Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return h_iOS15_1Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - a: 1 by 3 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as h_iOS15_1Output
    */
    public func prediction(a: MLMultiArray) throws -> h_iOS15_1Output {
        let input_ = h_iOS15_1Input(a: a)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - a: 1 by 3 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as h_iOS15_1Output
    */

    public func prediction(a: MLShapedArray<Float>) throws -> h_iOS15_1Output {
        let input_ = h_iOS15_1Input(a: a)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [h_iOS15_1Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [h_iOS15_1Output]
    */
    public func predictions(inputs: [h_iOS15_1Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [h_iOS15_1Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [h_iOS15_1Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  h_iOS15_1Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
