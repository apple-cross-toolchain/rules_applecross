//
// Zrow.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class ZrowInput : MLFeatureProvider {

    /// items as dictionary of 64-bit integers to doubles
    public var items: [Int64 : Double]

    /// k as optional integer value
    public var k: Int64? = nil

    public var featureNames: Set<String> { ["items", "k"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "items" {
            return try! MLFeatureValue(dictionary: items as [NSObject : NSNumber])
        }
        if featureName == "k" {
            return k == nil ? nil : MLFeatureValue(int64: k!)
        }
        return nil
    }

    public init(items: [Int64 : Double], k: Int64? = nil) {
        self.items = items
        self.k = k
    }

}


/// Model Prediction Output Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class ZrowOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// recs as sequence of 64-bit integers
    public var recs: [Int64] {
        provider.featureValue(for: "recs")!.sequenceValue!.int64Values as! [Int64]
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(recs: [Int64]) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["recs" : MLFeatureValue(sequence: MLSequence(int64s: recs as [NSNumber]))])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class Zrow {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Zrow", withExtension:"mlmodelc")!
    }

    /**
        Construct Zrow instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Zrow.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Zrow.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct Zrow instance by automatically loading the model from the app's bundle.
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
        Construct Zrow instance with explicit path to mlmodelc file
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
        Construct Zrow instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Zrow, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Zrow instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Zrow {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Zrow instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Zrow, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Zrow(model: model)))
            }
        }
    }

    /**
        Construct Zrow instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Zrow {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Zrow(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ZrowInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ZrowOutput
    */
    public func prediction(input: ZrowInput) throws -> ZrowOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ZrowInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ZrowOutput
    */
    public func prediction(input: ZrowInput, options: MLPredictionOptions) throws -> ZrowOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return ZrowOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ZrowInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ZrowOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: ZrowInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> ZrowOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return ZrowOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - items: dictionary of 64-bit integers to doubles
            - k: optional integer value

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ZrowOutput
    */
    public func prediction(items: [Int64 : Double], k: Int64?) throws -> ZrowOutput {
        let input_ = ZrowInput(items: items, k: k)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [ZrowInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [ZrowOutput]
    */
    public func predictions(inputs: [ZrowInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [ZrowOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [ZrowOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  ZrowOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
