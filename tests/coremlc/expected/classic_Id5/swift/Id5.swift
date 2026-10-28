//
// Id5.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
public class Id5Input : MLFeatureProvider {

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

    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public convenience init(x: MLShapedArray<Double>) {
        self.init(x: MLMultiArray(x))
    }

}


/// Model Prediction Output Type
@available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
public class Id5Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// y as 3 element vector of doubles
    public var y: MLMultiArray {
        provider.featureValue(for: "y")!.multiArrayValue!
    }

    /// y as 3 element vector of doubles
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
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
@available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
public class Id5 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Id5", withExtension:"mlmodelc")!
    }

    /**
        Construct Id5 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Id5.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Id5.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct Id5 instance with explicit path to mlmodelc file
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
        Construct Id5 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Id5, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Id5 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Id5 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Id5 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Id5, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Id5(model: model)))
            }
        }
    }

    /**
        Construct Id5 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Id5 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Id5(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Id5Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Id5Output
    */
    public func prediction(input: Id5Input) throws -> Id5Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Id5Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Id5Output
    */
    public func prediction(input: Id5Input, options: MLPredictionOptions) throws -> Id5Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return Id5Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Id5Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Id5Output
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: Id5Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> Id5Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return Id5Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 3 element vector of doubles

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Id5Output
    */
    public func prediction(x: MLMultiArray) throws -> Id5Output {
        let input_ = Id5Input(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: 3 element vector of doubles

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Id5Output
    */

    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public func prediction(x: MLShapedArray<Double>) throws -> Id5Output {
        let input_ = Id5Input(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [Id5Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [Id5Output]
    */
    public func predictions(inputs: [Id5Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [Id5Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [Id5Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  Id5Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
