from django.shortcuts import render, redirect
from .models import UserProfile
from .forms import UserProfileForm
from django.contrib.auth.decorators import login_required

@login_required
def edit_profile(request):
    profile, created = UserProfile.objects.get_or_create(user=request.user)
    if request.method == 'POST':
        if 'remove_nickname' in request.POST:
            profile.nickname = ""
            profile.save()
            return redirect('edit_profile')
        form = UserProfileForm(request.POST, instance=profile)  # Usunięto request.FILES
        if form.is_valid():
            form.save()
            return redirect('edit_profile')
    else:
        form = UserProfileForm(instance=profile)
    return render(request, 'userProfile/edit_profile.html', {'form': form})
