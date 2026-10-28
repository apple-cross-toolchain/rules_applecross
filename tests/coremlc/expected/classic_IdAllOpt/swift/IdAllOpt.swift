//
// IdAllOpt.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class IdAllOptInput : MLFeatureProvider {

    /// o as optional integer value
    public var o: Int64? = nil

    /// p as optional string value
    public var p: String? = nil

    public var featureNames: Set<String> { ["o", "p"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "o" {
            return o == nil ? nil : MLFeatureValue(int64: o!)
        }
        if featureName == "p" {
            return p == nil ? nil : MLFeatureValue(string: p!)
        }
        return nil
    }

    public init(o: Int64? = nil, p: String? = nil) {
        self.o = o
        self.p = p
    }

}


/// Model Prediction Output Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class IdAllOptOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// y as double value
    public var y: Double {
        provider.featureValue(for: "y")!.doubleValue
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(y: Double) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["y" : MLFeatureValue(double: y)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class IdAllOpt {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "IdAllOpt", withExtension:"mlmodelc")!
    }

    /**
        Construct IdAllOpt instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of IdAllOpt.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `IdAllOpt.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct IdAllOpt instance by automatically loading the model from the app's bundle.
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
        Construct IdAllOpt instance with explicit path to mlmodelc file
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
        Construct IdAllOpt instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<IdAllOpt, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct IdAllOpt instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> IdAllOpt {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct IdAllOpt instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<IdAllOpt, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(IdAllOpt(model: model)))
            }
        }
    }

    /**
        Construct IdAllOpt instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> IdAllOpt {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return IdAllOpt(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as IdAllOptInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as IdAllOptOutput
    */
    public func prediction(input: IdAllOptInput) throws -> IdAllOptOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as IdAllOptInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as IdAllOptOutput
    */
    public func prediction(input: IdAllOptInput, options: MLPredictionOptions) throws -> IdAllOptOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return IdAllOptOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as IdAllOptInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as IdAllOptOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: IdAllOptInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> IdAllOptOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return IdAllOptOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - o: optional integer value
            - p: optional string value

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as IdAllOptOutput
    */
    public func prediction(o: Int64?, p: String?) throws -> IdAllOptOutput {
        let input_ = IdAllOptInput(o: o, p: p)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [IdAllOptInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [IdAllOptOutput]
    */
    public func predictions(inputs: [IdAllOptInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [IdAllOptOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [IdAllOptOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  IdAllOptOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
