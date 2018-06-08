import React, { Component } from 'react';
import './App.css';
import ImageSlot from './ImageSlot.js';
import ImageEditor from './ImageEditor.js';
import {TagList, defaultTagOrdering} from './TagList.js';
import TifariAPI from "./APIComms.js";
import {ldebug, assert} from "./Logging.js";

class StateMutator {
    constructor(app, oldState) {
        this.newState = {};

        this.oldState = oldState;
        this.app = app;
    }

    getFinalState() {
        return this.newState;
    }

    getOldState() {
        return this.oldState;
    }

    renderImageList() {
        ldebug("Rendering image list");

        let images = this.getQueriedImagesAndMarkMutated();

        let mappedImages = images.map(img => 
            <ImageSlot 
                img={img} 
                key={img.id} 
                onClick={() => this.app.onSelectImage(img)}
            />
        );

        this.newState.queriedImagesAsElements = mappedImages;

        return this;
    }

    setTags(tags) {
        ldebug("Setting tags list to");
        ldebug(tags);

        this.newState.tags = tags;
        return this;
    }

    setTagOrdering(ordering) {
        this.newState.tagOrdering = ordering;
        return this;
    }

    orderTags() {
        
        if(!this.newState.hasOwnProperty("tags")) {
            this.newState.tags = this.oldState.tags;
        }

        let tagOrdering = this.newState.hasOwnProperty("tagOrdering")
            ? this.newState.tagOrdering
            : this.oldState.tagOrdering;

        tagOrdering.order(this.newState.tags);
    }

    setImageList(images) {
        ldebug("Setting image list.");
        ldebug(images);

        this.newState.queriedImages = images;
        return this;
    }

    addImageToList(image) {
        ldebug("Adding image to list");
        ldebug(image);

        let images = this.getQueriedImagesAndMarkMutated();

        // avoid duplicate images
        if(images.findIndex(i => i.id === image.id) !== -1)
            return this;
    
        images.push(image);

        return this;
    }

    getQueriedImagesAndMarkMutated() {
        if(!this.newState.hasOwnProperty("queriedImages")) {
            // if we copy this, we're going to have to add a second layer of
            // indirection by making this queriedImage array store
            // references to cached values within a cachce area somewhere
            this.newState.queriedImages = this.oldState.queriedImages;
        }

        return this.newState.queriedImages;
    }

    // doesn't update the image list.
    removeImageFromList(image) {
        ldebug("Removing image from list.");
        ldebug(image);

        let images = this.getQueriedImagesAndMarkMutated();
    
        let imgIndex = images.findIndex(i => i.id === image.id);
        if(imgIndex === -1) return this;

        images.splice(imgIndex, 1);

        return this;
    }

    // doesn't update the image list.
    removeTagFromImage(image, tag) {
        ldebug("Removing tag from image");
        ldebug(image);
        ldebug(tag);

        let tagIndex = image.tags.findIndex(t => t.id === tag.id);
        if(tagIndex === -1) return this;

        image.tags.splice(tagIndex, 1);

        return this;
    }

    // doesn't update the image list.
    // image must be a part of the app state
    addTagToImage(image, tag) {
        ldebug("Adding tag to image");
        ldebug(image);
        ldebug(tag);

        let tagIndex = image.tags.findIndex(t => t.id === tag.id);

        if(tagIndex === -1) {
            image.tags.push(tag);
        }

        return this;
    }

    getSelectedImages() {
        if(!this.newState.hasOwnProperty("selectedImages")) {
            ldebug("Marked selected images as dirty.");
            this.newState.selectedImages = this.oldState.selectedImages;
        }

        return this.newState.selectedImages;

    }

    addSelectedImage(image) {
        ldebug("Adding selected image");
        ldebug(image);

        let images = this.getSelectedImages();

        // avoid duplicate images
        if(images.findIndex(i => i.id === image.id) !== -1)
            return this;

        images.push(image);
        return this;
    }

    removeSelectedImage(image) {
        ldebug("Removing selected image");
        ldebug(image);

        let images = this.getSelectedImages();
        let imgIndex = images.findIndex(i => i.id == image.id);

        if(imgIndex === -1) 
            return this;
        
        images.splice(imgIndex, 1);

        return this;
    }

    clearSelectedImages() {
        ldebug("Clearing selected images");

        this.newState.selectedImages = [];
        return this;
    }

    setSearchTags(tagsArray) {
        ldebug("Setting search tags");
        ldebug(tagsArray);

        this.newState.searchTagNames = tagsArray;
        return this;
    }

    setIsInToBeTaggedList(state) {
        ldebug("Setting is in to be tagged list state to");
        ldebug(state);

        this.newState.isInToTagList = state;
        return this;
    }

    setTagListDisplayState(state) {
        ldebug("Setting tag list display state to");
        ldebug(state);

        this.newState.displayTagList = state;
        return this;
    }

    setToBeTaggedListSize(size) {
        ldebug("Seting to be tagged list size to");
        ldebug(size);

        this.newState.tagQueueSize = size;
        return this;
    }
}

class App extends Component {

    constructor(props) {
        super(props);

        this.state = {
            queriedImages: [],
            queriedImagesAsElements: [],
            isInToTagList: false,
            selectedImages: [],
            displayTagList: false,
            tagQueueSize: 0,
            searchTagNames: [],
            tags: [],
            tagOrdering: defaultTagOrdering,
        };

        this.refSearchBar = React.createRef();

        this.foreignRemoveImageFromSelected = this.foreignRemoveImageFromSelected.bind(this);
        this.foreignSetTagListOrdering      = this.foreignSetTagListOrdering.bind(this);
        this.foreignToggleTagListDisplay    = this.foreignToggleTagListDisplay.bind(this);
        this.foreignEscKeyListener          = this.foreignEscKeyListener.bind(this);
        this.foreignViewToBeTaggedList      = this.foreignViewToBeTaggedList.bind(this);

        this.foreignOnEditorRemoveTagFromSelected = this.foreignOnEditorRemoveTagFromSelected.bind(this);
        this.foreignOnEditorAddTagToSelected = this.foreignOnEditorAddTagToSelected.bind(this);
        this.foreignAddTagToSearch = this.foreignAddTagToSearch.bind(this);
    }

    hideEditorSidebar() {
        this.mutateState(mut => mut.clearSelectedImages());
    }

    foreignEscKeyListener(ev) {
        if(ev.key === "Escape") {
            this.hideEditorSidebar();
        }
    }

    componentWillMount() {
        this.updateToBeTaggedListSize();
        this.updateTagList();
    }

    componentDidMount() {
        document.addEventListener("keypress", this.foreignEscKeyListener, false);
    }

    componentWillUnmount() {
        document.removeEventListener("keypress", this.foreignEscKeyListener, false);
    }

    mutateState(lambda) {
        this.setState(oldState => {
            let mutator = new StateMutator(this, oldState);
            lambda(mutator);
            return mutator.getFinalState();
        });
    }

    // callback that's called when we want to select an image
    onSelectImage(img) {
        this.mutateState(mut => {
            mut.addSelectedImage(img);
        })
    }

    // callback that's called when we want to search the backend for tags
    doImageSearch(query) {
        let tags = query.split(" ");

        TifariAPI.search(tags)
            .then(images =>
                this.mutateState(mut =>
                    mut.setSearchTags(tags)
                       .setImageList(images)
                       .renderImageList()
                       .setIsInToBeTaggedList(false))
                );
    }

    updateToBeTaggedListSize() {
        TifariAPI.getTagQueueSize().then(size => this.mutateState(mut => mut.setToBeTaggedListSize(size)));
    }

    foreignSetTagListOrdering(ordering) {
        this.mutateState(mut => 
            mut.setTagOrdering(ordering)
               .orderTags()
        );
    }

    updateTagList() {
        TifariAPI.getAllTags().then(tags => { 
            tags.sort((a, b) => a.times_used < b.times_used);
            this.mutateState(mut => 
                mut.setTags(tags)
                   .orderTags()
            )
        });
    }

    doTagsMatchSearch(tags) {
        // searchTagNames and tags should intersect.

        const intersection = this.state.searchTagNames.filter(
            tagName => tags.findIndex(tag => tag.name === tagName) !== -1);

        ldebug("Intersecting tags:");
        ldebug(tags);

        return intersection.length > 0;
    }

    // callback that's called when we remove a tag from an image
    foreignOnEditorRemoveTagFromSelected(tag) {

        let imageIds = this.state.selectedImages.map(img => img.id);

        TifariAPI.removeTags([tag.id], imageIds);

        this.mutateState(mut => {

            let rerender = false;
            mut.getOldState().selectedImages
                .forEach(image => {

                    mut.removeTagFromImage(image, tag);

                    if(mut.getOldState().isInToTagList && 0 >= image.tags.length) {
                        mut.addImageToList(image);
                    } else if(!this.doTagsMatchSearch(image.tags)) {
                        mut.removeImageFromList(image);
                        rerender = true;
                    }

                });
    
            if(rerender);
               mut.renderImageList();
        });

        this.updateTagList();
        this.updateToBeTaggedListSize();
    }
    
    // callback that's called whenever we add a tag to an image
    foreignOnEditorAddTagToSelected(tagNames) { 

        let imageIds = this.state.selectedImages.map(img => img.id);

        TifariAPI.addTags(tagNames, imageIds)
            .then(tags => this.mutateState(mut => {

                // add each tag to each image
                tags.forEach(
                    tag => mut.getOldState().selectedImages.forEach(
                        img => mut.addTagToImage(img, tag))
                );

                let rerender = false;

                // if we're in the to tag list, remove all the selected images
                // since we just added tags to those images.
                if(mut.getOldState().isInToTagList) {
                    mut.getOldState().selectedImages
                        .forEach(img => mut.removeImageFromList(img));
                    rerender = true;
                } else {
                    mut.getOldState().selectedImages.forEach(img =>{
                        if(this.doTagsMatchSearch(img.tags)) {
                            mut.addImageToList(img);
                            rerender = true;
                        }
                    });
                }

                if(rerender)
                    mut.renderImageList();
            })
        );

        this.updateTagList();
        this.updateToBeTaggedListSize();
    }

    foreignToggleTagListDisplay() {
        this.mutateState(mut => mut.setTagListDisplayState(!mut.getOldState().displayTagList));
    }

    foreignViewToBeTaggedList() {
        TifariAPI.getToBeTaggedList()
            .then(images =>
                this.mutateState(mut => 
                    mut.setImageList(images)
                        .renderImageList()
                        .setIsInToBeTaggedList(true))
            );
    }

    foreignAddTagToSearch(tag) {

        if(this.state.searchTagNames.findIndex(t => t === tag.name) !== -1)
            return;

        let search = this.refSearchBar.current;
        search.value = search.value.trim();

        if(0 >= search.value.length) {
            search.value = search.value.concat(tag.name);
        } else {
            search.value = search.value.concat(" ", tag.name);
        }

        this.doImageSearch(search.value);
    }

    foreignRemoveImageFromSelected(image) {
        this.mutateState(mut => mut.removeSelectedImage(image));
    }

    render() {

        ldebug("Rendering");

        return (
            <div className="App">
                {this.state.selectedImages.length > 0 &&
                    <ImageEditor 
                        images = {this.state.selectedImages}
                        onAddTag = {this.foreignOnEditorAddTagToSelected}
                        onRemoveTag = {this.foreignOnEditorRemoveTagFromSelected}
                        callbackRemoveImageFromSelected = {this.foreignRemoveImageFromSelected}
                    />
                }

                {this.state.displayTagList &&
                    <TagList 
                        tags = {this.state.tags}
                        callbackSetOrdering = {this.foreignSetTagListOrdering}
                        callbackAddTag = {this.foreignAddTagToSearch}
                    />
                }

                <header>

                    <button onClick={this.foreignViewToBeTaggedList}>
                        View To-Tag List({this.state.tagQueueSize})
                    </button>

                    <button onClick={() => TifariAPI.reloadRoot()}>
                        Reload images
                    </button>
            
                    <button onClick={this.foreignToggleTagListDisplay}>
                        {this.state.displayTagList ? "Hide" : "Show"} tag list
                    </button>

                    <input
                        type = "text"
                        ref = {this.refSearchBar}
                        onChange = {ev => this.doImageSearch(ev.target.value.trim())}
                    />
                </header>

                <ul>{this.state.queriedImagesAsElements}</ul>
            </div>
        );
    }
}

export default App;
