//
// VScene2.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, visionOS 1.0, *)
@available(watchOS, unavailable)
public class VScene2Input : MLFeatureProvider {

    /// image as color (kCVPixelFormatType_32BGRA) image buffer, 360 pixels wide by 360 pixels high
    public var image: CVPixelBuffer

    public var featureNames: Set<String> { ["image"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "image" {
            return MLFeatureValue(pixelBuffer: image)
        }
        return nil
    }

    public init(image: CVPixelBuffer) {
        self.image = image
    }

    public convenience init(imageWith image: CGImage) throws {
        self.init(image: try MLFeatureValue(cgImage: image, pixelsWide: 360, pixelsHigh: 360, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!)
    }

    public convenience init(imageAt image: URL) throws {
        self.init(image: try MLFeatureValue(imageAt: image, pixelsWide: 360, pixelsHigh: 360, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!)
    }

    public func setImage(with image: CGImage) throws  {
        self.image = try MLFeatureValue(cgImage: image, pixelsWide: 360, pixelsHigh: 360, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

    public func setImage(with image: URL) throws  {
        self.image = try MLFeatureValue(imageAt: image, pixelsWide: 360, pixelsHigh: 360, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, visionOS 1.0, *)
@available(watchOS, unavailable)
public class VScene2Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// features as 768 element vector of floats
    public var features: MLMultiArray {
        provider.featureValue(for: "features")!.multiArrayValue!
    }

    /// features as 768 element vector of floats
    public var featuresShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(features)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(features: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["features" : MLFeatureValue(multiArray: features)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 14.0, iOS 17.0, tvOS 17.0, visionOS 1.0, *)
@available(watchOS, unavailable)
public class VScene2 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "VScene2", withExtension:"mlmodelc")!
    }

    /**
        Construct VScene2 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of VScene2.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `VScene2.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct VScene2 instance with explicit path to mlmodelc file
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
        Construct VScene2 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<VScene2, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct VScene2 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> VScene2 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct VScene2 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<VScene2, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(VScene2(model: model)))
            }
        }
    }

    /**
        Construct VScene2 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> VScene2 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return VScene2(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as VScene2Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as VScene2Output
    */
    public func prediction(input: VScene2Input) throws -> VScene2Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as VScene2Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as VScene2Output
    */
    public func prediction(input: VScene2Input, options: MLPredictionOptions) throws -> VScene2Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return VScene2Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as VScene2Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as VScene2Output
    */
    public func prediction(input: VScene2Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> VScene2Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return VScene2Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - image: color (kCVPixelFormatType_32BGRA) image buffer, 360 pixels wide by 360 pixels high

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as VScene2Output
    */
    public func prediction(image: CVPixelBuffer) throws -> VScene2Output {
        let input_ = VScene2Input(image: image)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [VScene2Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [VScene2Output]
    */
    public func predictions(inputs: [VScene2Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [VScene2Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [VScene2Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  VScene2Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
