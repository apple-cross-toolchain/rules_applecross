//
// Recommender.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class RecommenderInput : MLFeatureProvider {

    /// The list of items used to generate the recommendations. as dictionary of strings to doubles
    public var items: [String : Double]

    /// The number of items to return on a recommendation. as integer value
    public var k: Int64

    /// A sequence of items from which to generate recommendations. as optional sequence of strings
    public var restrict_: [String]? = nil

    /// A sequence of items to exclude from recommendations.  Defaults to the input item list if not given. as optional sequence of strings
    public var exclude: [String]? = nil

    public var featureNames: Set<String> { ["items", "k", "restrict", "exclude"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "items" {
            return try! MLFeatureValue(dictionary: items as [NSObject : NSNumber])
        }
        if featureName == "k" {
            return MLFeatureValue(int64: k)
        }
        if featureName == "restrict" {
            return restrict_ == nil ? nil : MLFeatureValue(sequence: MLSequence(strings: restrict_!))
        }
        if featureName == "exclude" {
            return exclude == nil ? nil : MLFeatureValue(sequence: MLSequence(strings: exclude!))
        }
        return nil
    }

    public init(items: [String : Double], k: Int64, restrict_: [String]? = nil, exclude: [String]? = nil) {
        self.items = items
        self.k = k
        self.restrict_ = restrict_
        self.exclude = exclude
    }

}


/// Model Prediction Output Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class RecommenderOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// The recommended items in order from most relevant to least relevant. as sequence of strings
    public var recommendations: [String] {
        provider.featureValue(for: "recommendations")!.sequenceValue!.stringValues
    }

    /// The scores for the recommended items, given as a dictionary of items and the corresponding scores. as dictionary of strings to doubles
    public var scores: [String : Double] {
        provider.featureValue(for: "scores")!.dictionaryValue as! [String : Double]
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(recommendations: [String], scores: [String : Double]) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["recommendations" : MLFeatureValue(sequence: MLSequence(strings: recommendations)), "scores" : MLFeatureValue(dictionary: scores as [AnyHashable : NSNumber])])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class Recommender {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Recommender", withExtension:"mlmodelc")!
    }

    /**
        Construct Recommender instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Recommender.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Recommender.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct Recommender instance by automatically loading the model from the app's bundle.
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
        Construct Recommender instance with explicit path to mlmodelc file
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
        Construct Recommender instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Recommender, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Recommender instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Recommender {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Recommender instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Recommender, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Recommender(model: model)))
            }
        }
    }

    /**
        Construct Recommender instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Recommender {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Recommender(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as RecommenderInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as RecommenderOutput
    */
    public func prediction(input: RecommenderInput) throws -> RecommenderOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as RecommenderInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as RecommenderOutput
    */
    public func prediction(input: RecommenderInput, options: MLPredictionOptions) throws -> RecommenderOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return RecommenderOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as RecommenderInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as RecommenderOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: RecommenderInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> RecommenderOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return RecommenderOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - items: The list of items used to generate the recommendations. as dictionary of strings to doubles
            - k: The number of items to return on a recommendation. as integer value
            - restrict_: A sequence of items from which to generate recommendations. as optional sequence of strings
            - exclude: A sequence of items to exclude from recommendations.  Defaults to the input item list if not given. as optional sequence of strings

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as RecommenderOutput
    */
    public func prediction(items: [String : Double], k: Int64, restrict_: [String]?, exclude: [String]?) throws -> RecommenderOutput {
        let input_ = RecommenderInput(items: items, k: k, restrict_: restrict_, exclude: exclude)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [RecommenderInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [RecommenderOutput]
    */
    public func predictions(inputs: [RecommenderInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [RecommenderOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [RecommenderOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  RecommenderOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
