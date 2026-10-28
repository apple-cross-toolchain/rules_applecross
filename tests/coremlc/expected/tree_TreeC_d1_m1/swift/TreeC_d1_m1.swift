//
// TreeC_d1_m1.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.13, iOS 11.0, tvOS 11.0, watchOS 4.0, visionOS 1.0, *)
public class TreeC_d1_m1Input : MLFeatureProvider {

    /// a as double value
    public var a: Double

    /// c as integer value
    public var c: Int64

    public var featureNames: Set<String> { ["a", "c"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "a" {
            return MLFeatureValue(double: a)
        }
        if featureName == "c" {
            return MLFeatureValue(int64: c)
        }
        return nil
    }

    public init(a: Double, c: Int64) {
        self.a = a
        self.c = c
    }

}


/// Model Prediction Output Type
@available(macOS 10.13, iOS 11.0, tvOS 11.0, watchOS 4.0, visionOS 1.0, *)
public class TreeC_d1_m1Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// y as string value
    public var y: String {
        provider.featureValue(for: "y")!.stringValue
    }

    /// yProbability as dictionary of strings to doubles
    public var yProbability: [String : Double] {
        provider.featureValue(for: "yProbability")!.dictionaryValue as! [String : Double]
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(y: String, yProbability: [String : Double]) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["y" : MLFeatureValue(string: y), "yProbability" : MLFeatureValue(dictionary: yProbability as [AnyHashable : NSNumber])])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 10.13, iOS 11.0, tvOS 11.0, watchOS 4.0, visionOS 1.0, *)
public class TreeC_d1_m1 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "TreeC_d1_m1", withExtension:"mlmodelc")!
    }

    /**
        Construct TreeC_d1_m1 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of TreeC_d1_m1.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `TreeC_d1_m1.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct TreeC_d1_m1 instance by automatically loading the model from the app's bundle.
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
    @available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
    public convenience init(configuration: MLModelConfiguration) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct TreeC_d1_m1 instance with explicit path to mlmodelc file
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
    @available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {
        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))
    }

    /**
        Construct TreeC_d1_m1 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<TreeC_d1_m1, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct TreeC_d1_m1 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> TreeC_d1_m1 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct TreeC_d1_m1 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<TreeC_d1_m1, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(TreeC_d1_m1(model: model)))
            }
        }
    }

    /**
        Construct TreeC_d1_m1 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> TreeC_d1_m1 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return TreeC_d1_m1(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as TreeC_d1_m1Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as TreeC_d1_m1Output
    */
    public func prediction(input: TreeC_d1_m1Input) throws -> TreeC_d1_m1Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as TreeC_d1_m1Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as TreeC_d1_m1Output
    */
    public func prediction(input: TreeC_d1_m1Input, options: MLPredictionOptions) throws -> TreeC_d1_m1Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return TreeC_d1_m1Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as TreeC_d1_m1Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as TreeC_d1_m1Output
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: TreeC_d1_m1Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> TreeC_d1_m1Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return TreeC_d1_m1Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - a: double value
            - c: integer value

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as TreeC_d1_m1Output
    */
    public func prediction(a: Double, c: Int64) throws -> TreeC_d1_m1Output {
        let input_ = TreeC_d1_m1Input(a: a, c: c)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [TreeC_d1_m1Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [TreeC_d1_m1Output]
    */
    @available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
    public func predictions(inputs: [TreeC_d1_m1Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [TreeC_d1_m1Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [TreeC_d1_m1Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  TreeC_d1_m1Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
