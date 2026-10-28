//
// Tagger1.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
public class Tagger1Input : MLFeatureProvider {

    /// Input text as string value
    public var text: String

    public var featureNames: Set<String> { ["text"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "text" {
            return MLFeatureValue(string: text)
        }
        return nil
    }

    public init(text: String) {
        self.text = text
    }

}


/// Model Prediction Output Type
@available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
public class Tagger1Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// Token tags as sequence of strings
    public var labels: [String] {
        provider.featureValue(for: "labels")!.sequenceValue!.stringValues
    }

    /// Token lengths as sequence of 64-bit integers
    public var lengths: [Int64] {
        provider.featureValue(for: "lengths")!.sequenceValue!.int64Values as! [Int64]
    }

    /// Token locations as sequence of 64-bit integers
    public var locations: [Int64] {
        provider.featureValue(for: "locations")!.sequenceValue!.int64Values as! [Int64]
    }

    /// Tokens as sequence of strings
    public var tokens: [String] {
        provider.featureValue(for: "tokens")!.sequenceValue!.stringValues
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(labels: [String], lengths: [Int64], locations: [Int64], tokens: [String]) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["labels" : MLFeatureValue(sequence: MLSequence(strings: labels)), "lengths" : MLFeatureValue(sequence: MLSequence(int64s: lengths as [NSNumber])), "locations" : MLFeatureValue(sequence: MLSequence(int64s: locations as [NSNumber])), "tokens" : MLFeatureValue(sequence: MLSequence(strings: tokens))])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
public class Tagger1 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Tagger1", withExtension:"mlmodelc")!
    }

    /**
        Construct Tagger1 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Tagger1.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Tagger1.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct Tagger1 instance by automatically loading the model from the app's bundle.
    */
    @available(*, deprecated, message: "Use init(configuration:) instead and handle errors appropriately.")
    public convenience init() {
        try! self.init(contentsOf: type(of:self).urlOfModelInThisBundle)
    }

    /**
        Construct a model with configuration

        - parameters:
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(configuration: MLModelConfiguration) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Tagger1 instance with explicit path to mlmodelc file
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
        Construct Tagger1 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Tagger1, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Tagger1 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Tagger1 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Tagger1 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Tagger1, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Tagger1(model: model)))
            }
        }
    }

    /**
        Construct Tagger1 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Tagger1 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Tagger1(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Tagger1Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Tagger1Output
    */
    public func prediction(input: Tagger1Input) throws -> Tagger1Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Tagger1Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Tagger1Output
    */
    public func prediction(input: Tagger1Input, options: MLPredictionOptions) throws -> Tagger1Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return Tagger1Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Tagger1Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Tagger1Output
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: Tagger1Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> Tagger1Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return Tagger1Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - text: Input text as string value

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Tagger1Output
    */
    public func prediction(text: String) throws -> Tagger1Output {
        let input_ = Tagger1Input(text: text)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [Tagger1Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [Tagger1Output]
    */
    public func predictions(inputs: [Tagger1Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [Tagger1Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [Tagger1Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  Tagger1Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
