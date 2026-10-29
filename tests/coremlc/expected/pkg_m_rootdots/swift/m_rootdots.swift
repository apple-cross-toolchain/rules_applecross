//
// m_rootdots.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
public class m_rootdotsInput : MLFeatureProvider {

    /// orig b_24 as integer value
    public var b_24: Int64

    public var featureNames: Set<String> { ["b_24"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "b_24" {
            return MLFeatureValue(int64: b_24)
        }
        return nil
    }

    public init(b_24: Int64) {
        self.b_24 = b_24
    }

}


/// Model Prediction Output Type
@available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
public class m_rootdotsOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// orig probs as integer value
    public var probs: Int64 {
        provider.featureValue(for: "probs")!.int64Value
    }

    /// orig z0 as dictionary of 64-bit integers to doubles
    public var z0: [Int64 : Double] {
        provider.featureValue(for: "z0")!.dictionaryValue as! [Int64 : Double]
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(probs: Int64, z0: [Int64 : Double]) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["probs" : MLFeatureValue(int64: probs), "z0" : MLFeatureValue(dictionary: z0 as [AnyHashable : NSNumber])])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
public class m_rootdots {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "m_rootdots", withExtension:"mlmodelc")!
    }

    /**
        Construct m_rootdots instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of m_rootdots.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `m_rootdots.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct m_rootdots instance by automatically loading the model from the app's bundle.
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
        Construct m_rootdots instance with explicit path to mlmodelc file
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
        Construct m_rootdots instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<m_rootdots, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct m_rootdots instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> m_rootdots {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct m_rootdots instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<m_rootdots, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(m_rootdots(model: model)))
            }
        }
    }

    /**
        Construct m_rootdots instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> m_rootdots {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return m_rootdots(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as m_rootdotsInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as m_rootdotsOutput
    */
    public func prediction(input: m_rootdotsInput) throws -> m_rootdotsOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as m_rootdotsInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as m_rootdotsOutput
    */
    public func prediction(input: m_rootdotsInput, options: MLPredictionOptions) throws -> m_rootdotsOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return m_rootdotsOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as m_rootdotsInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as m_rootdotsOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: m_rootdotsInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> m_rootdotsOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return m_rootdotsOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - b_24: orig b_24 as integer value

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as m_rootdotsOutput
    */
    public func prediction(b_24: Int64) throws -> m_rootdotsOutput {
        let input_ = m_rootdotsInput(b_24: b_24)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [m_rootdotsInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [m_rootdotsOutput]
    */
    public func predictions(inputs: [m_rootdotsInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [m_rootdotsOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [m_rootdotsOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  m_rootdotsOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
