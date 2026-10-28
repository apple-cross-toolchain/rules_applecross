//
// Id10.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 16.0, iOS 19.0, tvOS 19.0, watchOS 12.0, visionOS 3.0, *)
public class Id10Input : MLFeatureProvider {

    /// x as 3 element vector of doubles
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

    public convenience init(x: MLShapedArray<Double>) {
        self.init(x: MLMultiArray(x))
    }

}


/// Model Prediction Output Type
@available(macOS 16.0, iOS 19.0, tvOS 19.0, watchOS 12.0, visionOS 3.0, *)
public class Id10Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// y as 3 element vector of doubles
    public var y: MLMultiArray {
        provider.featureValue(for: "y")!.multiArrayValue!
    }

    /// y as 3 element vector of doubles
    public var yShapedArray: MLShapedArray<Double> {
        MLShapedArray<Double>(y)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(y: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["y" : MLFeatureValue(multiArray: y)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 16.0, iOS 19.0, tvOS 19.0, watchOS 12.0, visionOS 3.0, *)
public class Id10 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Id10", withExtension:"mlmodelc")!
    }

    /**
        Construct Id10 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Id10.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Id10.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct Id10 instance with explicit path to mlmodelc file
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
        Construct Id10 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Id10, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Id10 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Id10 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Id10 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Id10, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Id10(model: model)))
            }
        }
    }

    /**
        Construct Id10 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Id10 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Id10(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Id10Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Id10Output
    */
    public func prediction(input: Id10Input) throws -> Id10Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Id10Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Id10Output
    */
    public func prediction(input: Id10Input, options: MLPredictionOptions) throws -> Id10Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return Id10Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Id10Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Id10Output
    */
    public func prediction(input: Id10Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> Id10Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return Id10Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 3 element vector of doubles

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Id10Output
    */
    public func prediction(x: MLMultiArray) throws -> Id10Output {
        let input_ = Id10Input(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 3 element vector of doubles

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Id10Output
    */

    public func prediction(x: MLShapedArray<Double>) throws -> Id10Output {
        let input_ = Id10Input(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [Id10Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [Id10Output]
    */
    public func predictions(inputs: [Id10Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [Id10Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [Id10Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  Id10Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
